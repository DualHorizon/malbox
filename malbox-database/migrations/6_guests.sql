CREATE TABLE "guests" (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    label character varying(255) NOT NULL,
    manager character varying(255) NOT NULL,
    started_on timestamp without time zone NOT NULL,
    shutdown_on timestamp without time zone,
    task_id integer NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (task_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

SELECT trigger_updated_at('"guests"');
