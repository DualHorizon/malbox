CREATE TYPE task_state AS ENUM (
    'pending',
    'initializing',
    'preparing_resources',
    'running',
    'stopping',
    'completed',
    'failed',
    'canceled'
);

CREATE TYPE machine_arch AS ENUM (
    'x86',
    'x64'
);

CREATE TYPE machine_platform AS ENUM (
    'windows',
    'linux'
);
