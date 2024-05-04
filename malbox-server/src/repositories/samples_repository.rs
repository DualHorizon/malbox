use anyhow::Context;
use sqlx::{query, query_as, FromRow, PgPool};

pub struct Sample {
    pub file_size: i64,
    pub file_type: String,
    pub md5: String,
    pub crc32: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub ssdeep: String,
}

#[derive(FromRow, Debug)]
pub struct SampleEntity {
    pub id: i64,
    pub file_size: i64,
    pub file_type: String,
    pub md5: String,
    pub crc32: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub ssdeep: String,
}

impl Default for SampleEntity {
    fn default() -> Self {
        SampleEntity {
            id: 1,
            file_size: 2048,
            file_type: String::from("Default SampleEntity"),
            md5: String::from("none"),
            crc32: String::from("none"),
            sha1: String::from("none"),
            sha256: String::from("none"),
            sha512: String::from("none"),
            ssdeep: String::from("none"),
        }
    }
}

pub async fn insert_sample(pool: &PgPool, sample: Sample) -> anyhow::Result<SampleEntity> {
    query_as!(
        SampleEntity,
        r#"
    INSERT into "samples" (file_size, file_type, md5, crc32, sha1, sha256, sha512, ssdeep)
    values ($1::bigint, $2::varchar, $3::varchar, $4::varchar, $5::varchar, $6::varchar, $7::varchar, $8::varchar)
    returning *
    "#,
        sample.file_size,
        sample.file_type,
        sample.md5,
        sample.crc32,
        sample.sha1,
        sample.sha256,
        sample.sha512,
        sample.ssdeep
    )
    .fetch_one(pool)
    .await
    .context("failed to insert sample")
}
