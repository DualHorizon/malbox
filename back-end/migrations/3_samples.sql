CREATE TABLE "samples" (
    id integer NOT NULL,
    file_size integer NOT NULL,
    file_type character varying(255) NOT NULL,
    md5 character varying(32) NOT NULL,
    crc32 character varying(8) NOT NULL,
    sha1 character varying(40) NOT NULL,
    sha256 character varying(64) NOT NULL,
    sha512 character varying(128) NOT NULL,
    ssdeep character varying(255),
    PRIMARY KEY (id)
);

SELECT trigger_updated_at('"samples"');
