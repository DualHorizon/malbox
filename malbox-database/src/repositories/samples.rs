use crate::error::{Result, SampleError};
use sqlx::{postgres::PgDatabaseError, query_as, FromRow, PgPool};

#[derive(Debug, Clone)]
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

#[derive(FromRow, Debug, Clone)]
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

pub async fn insert_sample(pool: &PgPool, sample: Sample) -> Result<SampleEntity> {
    match query_as!(
        SampleEntity,
        r#"
        INSERT INTO "samples" (file_size, file_type, md5, crc32, sha1, sha256, sha512, ssdeep)
        VALUES ($1::bigint, $2::varchar, $3::varchar, $4::varchar, $5::varchar, $6::varchar, $7::varchar, $8::varchar)
        RETURNING *
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
    {
        Ok(new_sample) => Ok(new_sample),
        Err(e) => {
            if let Some(db_error) = e.as_database_error() {
                let pg_error = db_error.downcast_ref::<PgDatabaseError>();
                    if pg_error.code() == "23505" {
                        let existing_sample = query_as!(
                            SampleEntity,
                            r#"
                            SELECT * FROM "samples"
                            WHERE md5 = $1 AND crc32 = $2 AND sha1 = $3 AND sha256 = $4 AND sha512 = $5
                            "#,
                            sample.md5,
                            sample.crc32,
                            sample.sha1,
                            sample.sha256,
                            sample.sha512
                        )
                        .fetch_one(pool)
                        .await
                        .map_err(|e| SampleError::FetchFailed { hash: sample.sha256, message: "Failed to fetch existing sample".to_string(), source: e })?;

                        return Ok(existing_sample);
                    }
                }

            Err(SampleError::InsertFailed { hash: "".to_string(), message: "".to_string(), source: e }.into())
        }
    }
}
