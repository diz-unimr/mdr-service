create table if not exists modules
(
    id               uuid not null,
    name             text not null,
    fdpg_cds_code    text not null,
    fdpg_cds_system  text not null,
    fdpg_cds_version text not null,
    version          text not null,
    primary key (id)
);

create table if not exists concepts
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
create index if not exists idx_concept_module_id on concepts (module_id);
create index if not exists idx_concept_parent_id on concepts (parent_id);
