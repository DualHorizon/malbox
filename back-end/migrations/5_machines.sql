CREATE TABLE "machines" (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    label character varying(255) NOT NULL,
    ip character varying(255) NOT NULL,
    platform character varying(255) NOT NULL,
    locked boolean NOT NULL,
    locked_changed_on timestamp without time zone,
    status character varying(255),
    status_changed_on timestamp without time zone,
    PRIMARY KEY (id)
);

SELECT trigger_updated_at('"machines"');
