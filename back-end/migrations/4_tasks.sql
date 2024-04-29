CREATE TABLE "tasks" (
    id integer NOT NULL,
    target text NOT NULL,
    category character varying(255) NOT NULL,
    timeout integer DEFAULT 0 NOT NULL,
    priority integer DEFAULT 1 NOT NULL,
    custom character varying(255),
    machine character varying(255),
    package character varying(255),
    options character varying(255),
    platform character varying(255),
    memory boolean NOT NULL,
    enforce_timeout boolean NOT NULL,
    added_on timestamp without time zone NOT NULL,
    started_on timestamp without time zone,
    completed_on timestamp without time zone,
    status status_type DEFAULT 'pending'::status_type NOT NULL,
    sample_id integer,
    PRIMARY KEY (id),
    FOREIGN KEY (sample_id) REFERENCES samples(id)
);

SELECT trigger_updated_at('"tasks"');
