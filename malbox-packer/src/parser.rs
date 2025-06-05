use crate::error::{Error, Result};
use crate::packer::templates::vars::VarType;
use crate::packer::templates::{Provisioner, Source, Template, TemplateDependencies, Variable};
use malbox_hcl_utils::{
    extractor::{extract_enum_validation, extract_string, extract_string_array, HclExtractor},
    parse, Block, Body, Structure,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub fn parse_template(content: &str) -> Result<Template> {
    let body = parse(content).map_err(|e| Error::HclParse(e.into()))?;
    let mut variables = HashMap::new();
    let mut sources = Vec::new();
    let mut provisioners = Vec::new();
    let mut dependencies = TemplateDependencies::default();
    let mut description = None;

    for structure in body.iter() {
        match structure {
            Structure::Block(block) => match block.identifier().as_ref() {
                "variable" => {
                    if let Some(var) = parse_variable(block)? {
                        if var.0 == "description" {
                            if let Some(default) = &var.1.default {
                                description = Some(default.clone());
                            }
                        }
                        variables.insert(var.0, var.1);
                    }
                }
                "source" => {
                    if let Some(source) = parse_source(block)? {
                        extract_source_dependencies(block, &mut dependencies)?;
                        sources.push(source);
                    }
                }
                "build" => {
                    extract_build_dependencies(block, &mut dependencies)?;
                }
                "provisioner" => {
                    if let Some(provisioner) = parse_provisioner(block)? {
                        extract_provisioner_dependencies(block, &mut dependencies)?;
                        provisioners.push(provisioner);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Ok(Template::builder()
        .name(String::new())
        .content(content.to_string())
        .variables(variables)
        .sources(sources)
        .provisioners(provisioners)
        .dependencies(dependencies)
        .maybe_description(description)
        .build())
}

pub fn extract_description_from_body(body: &Body) -> Option<String> {
    for structure in body.iter() {
        if let Structure::Block(block) = structure {
            if block.identifier() == "variable" {
                if let Some(var_name) = block.labels().first() {
                    if var_name.as_str() == "description" {
                        for attr in block.body().attributes() {
                            if attr.key() == "default" {
                                return extract_string(attr.expr());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn parse_variable(block: &Block) -> Result<Option<(String, Variable)>> {
    let var_name = block
        .labels()
        .first()
        .ok_or_else(|| Error::Template("Variable missing name".to_string()))?
        .as_str()
        .to_string();

    let extractor = HclExtractor::new(block);

    let mut var = Variable {
        var_type: VarType::String,
        default: None,
        description: None,
        required: true,
        enum_values: None,
        sensitive: false,
    };

    // Extract type
    if let Ok(Some(type_str)) = extractor.extract_optional::<String>("type") {
        var.var_type = type_str.as_str().into();
    }

    // Extract default value
    if let Some(attr) = block.body().attributes().find(|a| a.key() == "default") {
        var.default = extract_string(attr.expr());
        var.required = false;
    }

    // Extract description
    var.description = extractor.extract_optional("description").unwrap_or(None);

    // Extract sensitive flag
    var.sensitive = extractor.extract_with_default("sensitive", false);

    // Extract validation
    if let Some(attr) = block.body().attributes().find(|a| a.key() == "validation") {
        var.enum_values = extract_enum_validation(attr);
    }

    Ok(Some((var_name, var)))
}

pub fn parse_source(block: &Block) -> Result<Option<Source>> {
    let labels: Vec<_> = block.labels().into();
    if labels.len() < 2 {
        return Err(Error::Template("Invalid source block".to_string()));
    }

    let mut config = HashMap::new();
    for attr in block.body().attributes() {
        config.insert(attr.key().to_string(), attr.expr().to_string());
    }

    Ok(Some(Source {
        source_type: labels[0].as_str().to_string(),
        name: labels[1].as_str().to_string(),
        config,
    }))
}

pub fn parse_provisioner(block: &Block) -> Result<Option<Provisioner>> {
    let prov_type = block
        .labels()
        .first()
        .ok_or_else(|| Error::Template("Provisioner missing type".to_string()))?
        .as_str()
        .to_string();

    let mut config = HashMap::new();
    for attr in block.body().attributes() {
        config.insert(attr.key().to_string(), attr.expr().to_string());
    }

    Ok(Some(Provisioner {
        provisioner_type: prov_type,
        config,
    }))
}

pub fn extract_source_dependencies(block: &Block, deps: &mut TemplateDependencies) -> Result<()> {
    for attr in block.body().attributes() {
        match attr.key() {
            "http_directory" => {
                if let Some(dir) = extract_string(attr.expr()) {
                    deps.http_directories.insert(dir);
                }
            }
            "floppy_files" => {
                if let Ok(files) = extract_string_array(attr.expr()) {
                    for file in files {
                        if let Some(filename) = Path::new(&file).file_name() {
                            if let Some(name) = filename.to_str() {
                                deps.floppy_files.insert(name.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn extract_build_dependencies(block: &Block, deps: &mut TemplateDependencies) -> Result<()> {
    for structure in block.body().iter() {
        if let Structure::Block(inner_block) = structure {
            if inner_block.identifier() == "provisioner" {
                extract_provisioner_dependencies(inner_block, deps)?;
            }
        }
    }
    Ok(())
}

pub fn extract_provisioner_dependencies(
    block: &Block,
    deps: &mut TemplateDependencies,
) -> Result<()> {
    if let Some(provisioner_type) = block.labels().first() {
        match provisioner_type.as_str() {
            "shell" | "powershell" => {
                for attr in block.body().attributes() {
                    match attr.key() {
                        "scripts" => {
                            if let Ok(scripts) = extract_string_array(attr.expr()) {
                                for script in scripts {
                                    if let Some(filename) = Path::new(&script).file_name() {
                                        if let Some(name) = filename.to_str() {
                                            deps.script_files.insert(name.to_string());
                                        }
                                    }
                                }
                            }
                        }
                        "script" => {
                            if let Some(script) = extract_string(attr.expr()) {
                                if let Some(filename) = Path::new(&script).file_name() {
                                    if let Some(name) = filename.to_str() {
                                        deps.script_files.insert(name.to_string());
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            "ansible" => {
                for attr in block.body().attributes() {
                    if attr.key() == "playbook_file" {
                        if let Some(playbook) = extract_string(attr.expr()) {
                            if let Some(filename) = Path::new(&playbook).file_name() {
                                if let Some(name) = filename.to_str() {
                                    deps.provisioner_files.insert(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}
