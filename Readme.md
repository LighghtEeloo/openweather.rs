# OpenWeather CLI in Rust

Upon installation, please run

```
openweather init
```

and enter the `api_key` acquired from `https://openweathermap.org/api`.

There are two types of apis from `openweathermap.org`, and two modes of query:
1. version 3.0, requires sign-up and subscription, and supports both `mode = "city"` and `mode = "location"`; **`mode = location` is better because it allows detailed information.**
2. version 2.5, only requires sign-up, but correspondingly only supports `mode = "city"`.

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