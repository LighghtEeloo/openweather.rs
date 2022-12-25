# OpenWeather CLI in Rust

An **unofficial** cli for openweathermap that outputs json purely for api calling purposes.

Upon installation, please run

```
openweather init
```

and enter the `api_key` acquired from `https://openweathermap.org/api`.

There are two modes of query from `openweathermap.org`:
1. `mode = "location"` which is better because it allows detailed information;
2. `mode = "city"` which only supports current weather query.

The rest are the granularity settings.

You can always come back and config them with

```
openweather edit
```

The rest commands can be found by running plain `openweather`.

Upon querying, run

```
openweather query
```

and use `openweather query -h` for more information.