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

copy modules (id, name, fdpg_cds_code, fdpg_cds_system, fdpg_cds_version, version)
    from '/docker-entrypoint-initdb.d/modules.csv' DELIMITER ',' CSV HEADER;

create table concepts
(
    id                       uuid    not null,
    module_id                uuid    not null
        constraint concepts_modules_id_fk
            references modules,
    parent_id                uuid
        constraint concepts_concepts_id_fk
            references concepts,
    display                  text    not null,
    term_codes               jsonb,
    selectable               boolean not null,
    leaf                     boolean not null,
    time_restriction_allowed boolean,
    filter_type              text,
    filter_options           jsonb,
    version                  text    not null,
    primary key (id)
);
create index idx_concept_module_id on concepts (module_id);
create index idx_concept_parent_id on concepts (parent_id);

copy concepts (id, module_id, parent_id, display, term_codes, selectable, leaf, time_restriction_allowed, filter_type,
               filter_options, version)
    from '/docker-entrypoint-initdb.d/concepts.csv' DELIMITER ',' CSV HEADER;

