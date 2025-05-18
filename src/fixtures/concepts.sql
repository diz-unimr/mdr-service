insert into modules (id, name, fdpg_cds_code, fdpg_cds_system, fdpg_cds_version, version)
values ('4bfd4e2ecaf5f7ae3ef8400ab0858ec7', 'Laboruntersuchung', 'Laboruntersuchung', 'fdpg.mii.cds', '1.0.0', '2.2.0'),
       ('f6d13ed9f9a1dd6042ee01f8c924a586', 'Diagnose', 'Diagnose', 'fdpg.mii.cds', '1.0.0', '2.2.0');

insert into concepts (id, module_id, parent_id, display, term_codes, selectable, leaf, time_restriction_allowed,
                      filter_type, filter_options, version)
values ('6a0c97ad28afc3e3a8da9416e6936ce8', '4bfd4e2ecaf5f7ae3ef8400ab0858ec7', null, 'Medikamente', null, false, false,
        null, null, null, '2.2.0'),
       ('ce3e2ac86da74b367e7d57a628022aca', '4bfd4e2ecaf5f7ae3ef8400ab0858ec7', '6a0c97ad28afc3e3a8da9416e6936ce8',
        'Antibiotika', null, false, false, null, null, null, '2.2.0'),
       ('6f12427c7db35328e268206113ac1c69', '4bfd4e2ecaf5f7ae3ef8400ab0858ec7', 'ce3e2ac86da74b367e7d57a628022aca',
        'Voriconazol [Fremdlabor]',
        '[{"code": "VORI", "system": "https://fhir.diz.uni-marburg.de/CodeSystem/swisslab-code", "display": "Voriconazol [Fremdlabor]", "version": null}, {"code": "38370-3", "system": "http://loinc.org", "display": "Voriconazole [Mass/volume] in Serum or Plasma", "version": "2.42"}]',
        true, true, true, null, null, '2.2.0'),
       ('a52b18659011fe8adeb112ce01327a2d', '4bfd4e2ecaf5f7ae3ef8400ab0858ec7', 'ce3e2ac86da74b367e7d57a628022aca',
        'Vancomycin',
        '[{"code": "VANC", "system": "https://fhir.diz.uni-marburg.de/CodeSystem/swisslab-code", "display": "Vancomycin"}, {"code": "20578-1", "system": "http://loinc.org", "display": "Vancomycin [Mass/volume] in Serum or Plasma", "version": "2.73"}]',
        true, true, true, null, null, '2.2.0'),

       ('7ebd739d-d203-2fb4-7c78-8e753e69b507', 'f6d13ed9-f9a1-dd60-42ee-01f8c924a586', null,
        'Angeborene Fehlbildungen, Deformitäten und Chromosomenanomalien',
        '[{"code": "XVII", "system": "http://fhir.de/CodeSystem/bfarm/icd-10-gm", "display": "Angeborene Fehlbildungen, Deformitäten und Chromosomenanomalien", "version": "2024"}]',
        false, false, true, null, null, '2.2.0'),
       ('2999dc94-3086-b640-eb3e-d82b8dcea026', 'f6d13ed9-f9a1-dd60-42ee-01f8c924a586',
        '7ebd739d-d203-2fb4-7c78-8e753e69b507', 'Angeborene Fehlbildungen der Genitalorgane',
        '[{"code": "Q50-Q56", "system": "http://fhir.de/CodeSystem/bfarm/icd-10-gm", "display": "Angeborene Fehlbildungen der Genitalorgane", "version": "2024"}]',
        true, false, true, null, null, '2.2.0'),
       ('f8f46412-df1f-42ee-6eca-845452fa507d', 'f6d13ed9-f9a1-dd60-42ee-01f8c924a586',
        '2999dc94-3086-b640-eb3e-d82b8dcea026',
        'Angeborene Fehlbildungen der Ovarien, der Tubae uterinae und der Ligg. lata uteri',
        '[{"code": "Q50", "system": "http://fhir.de/CodeSystem/bfarm/icd-10-gm", "display": "Angeborene Fehlbildungen der Ovarien, der Tubae uterinae und der Ligg. lata uteri", "version": "2024"}]',
        true, false, true, null, null, '2.2.0');