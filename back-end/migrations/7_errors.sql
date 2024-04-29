CREATE TABLE "errors" (
    id integer NOT NULL,
    message character varying(255) NOT NULL,
    task_id integer NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (task_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

SELECT trigger_updated_at('"errors"');
