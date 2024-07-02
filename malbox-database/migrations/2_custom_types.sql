CREATE TYPE status_type AS ENUM (
    'pending',
    'processing',
    'failure',
    'success'
);

CREATE TYPE machine_arch AS ENUM (
    'x86',
    'x64'
);

CREATE TYPE machine_platform AS ENUM (
    'windows',
    'linux',
    'android',
    'macos'
);
