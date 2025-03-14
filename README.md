# 🏷️ mdr-service

[![MegaLinter](https://github.com/diz-unimr/mdr-service/actions/workflows/mega-linter.yml/badge.svg)](https://github.com/diz-unimr/mdr-service/actions/workflows/mega-linter.yml)
[![build](https://github.com/diz-unimr/mdr-service/actions/workflows/build.yaml/badge.svg)](https://github.com/diz-unimr/mdr-service/actions/workflows/build.yaml)
[![docker](https://github.com/diz-unimr/mdr-service/actions/workflows/release.yaml/badge.svg)](https://github.com/diz-unimr/mdr-service/actions/workflows/release.yaml)
[![codecov](https://codecov.io/gh/diz-unimr/mdr-service/graph/badge.svg?token=xrpWpysCri)](https://codecov.io/gh/diz-unimr/mdr-service)


> RESTful API for the Marburg metadata repository

## CDS Ontology API

MII Core Dataset ontology consisting of modules and concepts and their relations represented as a hierarchical
tree structure.

Ontology data is used by the Marburg feasibility portal (FDPM) to query local DIC FHIR 🔥 data.

### REST Endpoints

------------------------------------------------------------------------------------------

#### CDS modules (list, get single, create)

<details>
 <summary><code>GET</code> <code><b>/ontology/modules</b></code> <code>(get all CDS module data)</code></summary>

##### Parameters

> None

##### Responses

> | http code | content-type               | response                       |
> |-----------|----------------------------|--------------------------------|
> | `200`     | `application/json`         | Array of modules               |
> | `500`     | `text/plain;charset=UTF-8` | Error message                  |

##### Example cURL

> ```sh
>  curl -X GET http://localhost:3000/ontology/modules
> ```

</details>

<details>
 <summary><code>POST</code> <code><b>/modules</b></code> <code>(create CDS module)</code></summary>

##### Parameters

> None

##### Body

> | content-type       | data type     | required |
> |--------------------|---------------|----------|
> | `application/json` | Module object | true     |

##### Responses

> | http code | content-type               | response                           |
> |-----------|----------------------------|------------------------------------|
> | `201`     | `application/json`         | The id of the newly created module |
> | `500`     | `text/plain;charset=UTF-8` | Error message                      |

##### Example cURL

> ```sh
> curl -X POST -H "Content-Type: application/json" --data @payload.json http://localhost:3000/ontology/modules
> ```

</details>

<details>
  <summary><code>GET</code> <code><b>/ontology/modules/{id}</b></code> <code>(get CDS module by id)</code></summary>

##### Parameters

> | name |  type      | data type      | description                           |
> |------|------------|----------------|---------------------------------------|
> | `id` |  required  | string         | The module's unique identifier (uuid) |

##### Responses

> | http code | content-type               | response                       |
> |-----------|----------------------------|--------------------------------|
> | `200`     | `application/json`         | Module data                    |
> | `404`     | `text/plain;charset=UTF-8` | `No module found with id: xyz` |
> | `500`     | `text/plain;charset=UTF-8` | Error message                  |

##### Example cURL

> ```sh
>  curl -X GET http://localhost:3000/ontology/modules/xzy
> ```

</details>

------------------------------------------------------------------------------------------

#### Ontology tree and concept search

<details>
  <summary><code>GET</code> <code><b>/ontology/tree/{module_id}</b></code> <code>(get complete ontology concept tree by module id)</code></summary>

##### Parameters

> | name        |  type      | data type      | description                           |
> |-------------|------------|----------------|---------------------------------------|
> | `module_id` |  required  | string         | The module's unique identifier (uuid) |

##### Responses

> | http code | content-type               | response                                  |
> |-----------|----------------------------|-------------------------------------------|
> | `200`     | `application/json`         | Nested ontology concept tree by module_id |
> | `500`     | `text/plain;charset=UTF-8` | Error message                             |

##### Example cURL

> ```sh
>  curl -X GET http://localhost:3000/ontology/tree/xzy
> ```

</details>

<details>
  <summary><code>POST</code> <code><b>/ontology/concepts/search</b></code> <code>(search ontology concepts' display and code values by text)</code></summary>

##### Parameters

> None

##### Body

> | content-type       | data type                                                    | required |
> |--------------------|--------------------------------------------------------------|----------|
> | `application/json` | Search object `{"module_id": String, "search_term": String}` | true     |

##### Responses

> | http code | content-type               | response                                            |
> |-----------|----------------------------|-----------------------------------------------------|
> | `200`     | `application/json`         | Array of concepts matching the search term          |
> | `400`     | `text/plain;charset=UTF-8` | `Search term must consist of at least 2 characters` |
> | `500`     | `text/plain;charset=UTF-8` | Error message                                       |

##### Example cURL

> ```sh
> curl -X POST -H "Content-Type: application/json" --data @payload.json http://localhost:3000/ontology/concepts/search
> ```

</details>

## Configuration properties

Application properties are read from a properties file ([app.yaml](./app.yaml)) with default values.

| Name                       | Default | Description                             |
|----------------------------|---------|-----------------------------------------|
| `app.log_level`            | debug   | Log level (error,warn,info,debug,trace) |
| `database.url`             |         | Postgres database connection string     |
| `database.max_connections` |         | Max database connections                |
| `database.timeout`         |         | Database connection timeout in seconds  |

### Environment variables

Override configuration properties by providing environment variables with their respective property names.

## License

[AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html)