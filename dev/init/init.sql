create table modules
(
    id               uuid not null,
    name             text not null,
    fdpg_cds_code    text not null,
    fdpg_cds_system  text not null,
    fdpg_cds_version text not null,
    version          text not null,
    primary key (id)
);

copy modules(id,name,fdpg_cds_code,fdpg_cds_system,fdpg_cds_version,version)
from '/docker-entrypoint-initdb.d/modules.csv' DELIMITER ','  CSV HEADER;
