use crate::error::{Error, Result};
use crate::packer::templates::vars::VarType;
use crate::packer::templates::{Provisioner, Source, Template, TemplateDependencies, Variable};
use crate::parser::hcl_custom;
use hcl::{Block, Body};
use std::collections::{HashMap, HashSet};

pub fn parse_template(content: &str) -> Result<Template> {
    let body = hcl_custom::parse(content)?;
    let mut variables = HashMap::new();
    let mut sources = Vec::new();
    let mut provisioners = Vec::new();
    let mut dependencies = TemplateDependencies::default();
    let mut description = None;

    for structure in body.iter() {
        match structure {
            hcl::Structure::Block(block) => match block.identifier().as_ref() {
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
        if let hcl::Structure::Block(block) = structure {
            if block.identifier() == "variable" {
                if let Some(var_name) = block.labels().first() {
                    if var_name.as_str() == "description" {
                        for attr in block.body().attributes() {
                            if attr.key() == "default" {
                                return Some(attr.expr().to_string());
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

    let mut var = Variable {
        var_type: VarType::String,
        default: None,
        description: None,
        required: true,
        enum_values: None,
        sensitive: false,
    };

    for attr in block.body().attributes() {
        match attr.key() {
            "type" => var.var_type = attr.expr().to_string().as_str().into(),
            "default" => {
                var.default = Some(attr.expr().to_string());
                var.required = false;
            }
            "description" => var.description = Some(attr.expr().to_string()),
            "sensitive" => var.sensitive = attr.expr().to_string().parse().unwrap_or(false),
            "validation" => {
                if let Some(enum_values) = hcl_custom::parse_enum_validation(attr) {
                    var.enum_values = Some(enum_values);
                }
            }
            _ => {}
        }
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
                if let Some(dir) = hcl_custom::extract_string_value(attr.expr()) {
                    deps.http_directories.insert(dir);
                }
            }
            "floppy_files" => {
                if let hcl::Expression::Array(items) = attr.expr() {
                    for item in items {
                        if let Some(path) = hcl_custom::extract_string_value(item) {
                            if let Some(filename) = std::path::Path::new(&path).file_name() {
                                if let Some(name) = filename.to_str() {
                                    deps.floppy_files.insert(name.to_string());
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

pub fn extract_build_dependencies(block: &Block, deps: &mut TemplateDependencies) -> Result<()> {
    for structure in block.body().iter() {
        if let hcl::Structure::Block(inner_block) = structure {
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
                            if let hcl::Expression::Array(items) = attr.expr() {
                                for item in items {
                                    if let Some(path) = hcl_custom::extract_string_value(item) {
                                        if let Some(filename) =
                                            std::path::Path::new(&path).file_name()
                                        {
                                            if let Some(name) = filename.to_str() {
                                                deps.script_files.insert(name.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        "script" => {
                            if let Some(script) = hcl_custom::extract_string_value(attr.expr()) {
                                if let Some(filename) = std::path::Path::new(&script).file_name() {
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
                        if let Some(playbook) = hcl_custom::extract_string_value(attr.expr()) {
                            if let Some(filename) = std::path::Path::new(&playbook).file_name() {
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
