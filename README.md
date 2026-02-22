# Branchy

Небольшой DSL на деревьях: из программы случайно выбирается один «листовой» результат.

## Сборка и тесты (Docker Compose)

Перед run/compile всегда запускаются тесты:

```powershell
docker-compose run --rm test
```

Запуск скрипта (сначала тесты, затем выполнение):

```powershell
docker-compose run --rm app run examples/hello.branchy
docker-compose run --rm app run examples/hello.branchy [input] [--seed N]
```

- Без `--seed` результат случайный. С `--seed N` — детерминированный (один и тот же вывод при одном и том же seed).
- Запуск с входом для событий (event): `docker-compose run --rm app run examples/events.branchy start`

Компиляция в бинарный формат:

```powershell
docker-compose run --rm app compile examples/hello.branchy -o out.branchyc
docker-compose run --rm app run out.branchyc
```

## Форматтер

Форматтер приводит исходник `.branchy` к единому стилю: отступы (2 пробела), один `;` между элементами ветки, перенос длинных веток на несколько строк (порог 80 символов), единообразное оформление блоков и вызовов.

### CLI

```powershell
# Вывести отформатированный код в stdout (исходник — из файла или stdin)
docker-compose run --rm app fmt examples/hello.branchy
docker-compose run --rm app format examples/hello.branchy

# Перезаписать файл
docker-compose run --rm app fmt examples/hello.branchy -w
docker-compose run --rm app fmt examples/hello.branchy --write

# Проверить, что файл уже отформатирован (без изменений; exit 1, если нет)
docker-compose run --rm app fmt examples/hello.branchy --check
docker-compose run --rm app fmt examples/hello.branchy -c
```

- **Без флагов** — читает файл (или stdin), парсит, печатает результат. Файл не меняется.
- **`-w` / `--write`** — записывает результат обратно в указанный файл. Требуется путь (stdin с `-w` нельзя).
- **`-c` / `--check`** — проверка: если после форматирования вывод совпадает с исходником, exit 0; иначе печатает сообщение вида `"path is not formatted (run branchy fmt -w to fix)"` и exit 1. Удобно для CI.

При ошибке парсинга форматтер не меняет файл и завершается с ненулевым кодом.

### Скрипты (PowerShell)

В корне репозитория:

- **`.\format.ps1`** — отформатировать все `examples/*.branchy` (перезапись файлов). Вызывает `fmt_all` через docker-compose.
- **`.\check-format.ps1`** — проверить, что все примеры уже отформатированы; выход с кодом 1, если хотя бы один файл нужно править (удобно для CI). Вызывает `check_fmt` через docker-compose.

### Стиль по умолчанию

- Отступ: 2 пробела.
- Один символ `;` между элементами ветки, без trailing `;` перед `]`.
- Ветка в одну строку, если умещается в 80 символов; иначе — многострочно с переносами после `[` и перед `]`.
- Пробелы вокруг `+` и `*`.
- Пустая строка перед главной веткой, если есть include/функции/события.
- Блоки вызовов `{ :param = ...; }` с отступами и переносами.

Комментарии в языке не предусмотрены; при форматировании сохраняется только структура программы (парсер → AST → обратный вывод).

### API и веб

- **POST /api/format** — тело `{ "source": "…" }`, ответ `{ "formatted": "…" }` или `400` с ошибкой парсинга.
- В веб-интерфейсе (http://localhost:8081) кнопка **«Форматировать»** под редактором вызывает этот endpoint и подставляет отформатированный код в поле исходника.

## Синтаксис

- **Ветка**: `[ a; b; c; ]` — случайно выбирается один из `a`, `b`, `c`.
- **Шаблон с блоком**: `hello :who { :who = [ world; human; ]; }` — подстановка `:who`, результат например `hello world`.
- **Опциональный параметр** `:?var` — подставляется или пропускается (50/50), например `"привет " + :?user`.
- **Инлайн**: `hello <a|b|c>` — короткая запись вариантов.
- **Пользовательская функция**: `!greet(:x) = [ hi :x; bye :x; ]`, вызов `[ !greet(world); ]` → `hi world` или `bye world`.
- **Операторы**:
  - `"a" + "b"` — конкатенация.
  - `expr * n` — выражение `expr` вычисляется `n` раз, результаты склеиваются (каждый раз новый случайный выбор). Примеры: `"x" * 5`, `[ "y"; "x"; ] * 1..30`, `word <aa|bb|cc> * 10`.
  - Справа от `*` можно указать диапазон: `"x" * 1..3` — длина от 1 до 3.
- **Инлайн-блок символов**: `[a-zA-Z]` — один символ из набора; `[a-zA-Z:5]` — 5 символов; `[a-z:2..5]` — от 2 до 5. Диапазоны: `a-z`, `0-9`.
- **Встроенные функции**: `!upper(s)`, `!lower(s)`, `!trim(s)`, `!concat(a,b)`, `!join(sep,a,b,c)`, `!len(s)`, `!replace(s,from,to)`, `!split(s,sep)`.

### События (events)

Входная строка (аргумент при запуске или поле `input` в API) сопоставляется с объявленными событиями; выполняется ветка первого совпадения.

- Имя: `@myEvent = [ привет; пока; ]`
- Строка: `"привет" = [ hello; bye; ]`
- Регулярное выражение: `~"сер[ёе]жа" = [ match; ]`

Если вход не передан или не совпал ни с одним событием, выполняется основная ветка (main).

### Подключение файлов (include)

В начале программы:

```
include "lib.branchy";
```

Подключаются функции и события из указанного файла (путь относительно текущего скрипта). В API include не поддерживаются — нужно передавать уже объединённый исходник.

### Миксины в ветке

- **`...:var`** — подставить в ветку содержимое параметра из блока вызова. Работает в теле функции, если вызов был с блоком: `wrap :_ { :extra = [ x; y; ]; }`.
- **`...include "path"`** — подставить в ветку main-ветку из другого файла (разрешается при загрузке).

Пример: `[ a; b; ...:extra; c; ]` при `:extra = [ x; y; ]` даёт варианты `a`, `b`, `x`, `y`, `c`.

## Примеры (examples/)

| Файл | Описание |
|------|----------|
| **showcase.branchy** | **Один большой пример: события, ветки, +/\*, :?param, шаблоны с блоком, миксины, инлайн, char block, builtins** |
| big_blocks.branchy | Большой пример с переменными и блоками `:var = { ... }` (section, card, row, config, api, layout, query, greet, format) — удобен для замеров размер/скорость байткода |
| api_style.branchy | Шаблоны в стиле API: `!api(:method, :path)`, вызовы с блоками и без |
| hello.branchy | Простая ветка |
| inline.branchy | Инлайн-варианты `<a\|b\|c>` |
| functions.branchy | Функции, операторы + и * |
| strings.branchy | Строки, конкатенация, повтор, `*` к ветке/инлайну, диапазон `1..n` |
| nested.branchy | Вложенные ветки |
| call_with_block.branchy | Шаблоны с блоками |
| mixed.branchy | Всё вместе |
| templates.branchy | Шаблоны с `:_`, `:?opt`, функции `!title`, `!pair`, `!row`, `!greet` |
| optional_param.branchy | Опциональный параметр `:?var` (50/50 вывод) |
| optional_vars.branchy | Опциональные параметры в шаблонах (`!line`, `!format` с `:?b`, `:?c`) |
| math.branchy | Конкатенация и повтор строк (`+`, `*`) |
| with_include.branchy, lib.branchy | Include |
| events.branchy | События по входу |
| mixins.branchy | Миксин ...:var (блок вызова) |
| mixins_include.branchy, snippet.branchy | Миксин ...include "файл" |
| escaping.branchy | Экранирование в строках |
| char_block.branchy | Инлайн-блок символов `[a-zA-Z]`, `[abc:5]`, `[a-z:2..5]` |

## Веб-сервис (фронт + nginx)

React-интерфейс и API за nginx: фронт на порту 8081, запросы к `/api/` проксируются на бэкенд.

```powershell
docker-compose up gateway web
```

Откройте **http://localhost:8081** — форма с полем исходника, выбором примера, опциональными полями «Вход для события» и «Seed». Кнопка «Выполнить» шлёт POST на `/api/run`.

- **GET /api/health** — `200` и `ok`
- **GET /api/examples** — JSON-массив примеров `{ "id", "name", "source" }` (файлы из `examples/`).
- **POST /api/run** — JSON `{ "source": "…" }`, опционально `"input": "строка"`, `"seed": number`. Ответ `{ "result": "…" }` или `400` с телом `{ "error": "…", "line?", "column?", "end_line?", "end_column?" }`. Поля `line`/`column`/`end_line`/`end_column` при наличии — для подсветки ошибки во фронте. С `seed` результат воспроизводим.

Пример вызова API напрямую:

```powershell
Invoke-RestMethod -Uri http://localhost:8081/api/run -Method Post -ContentType "application/json" -Body '{"source":"[ hello; world; 42; ]"}'
```

С событием по входу:

```powershell
Invoke-RestMethod -Uri http://localhost:8081/api/run -Method Post -ContentType "application/json" -Body '{"source":"@go = [ ok; ]; [ default; ]","input":"go"}'
```

### Локальная разработка фронта

```powershell
cd frontend
npm install
npm run dev
```

Dev-сервер (порт 5173) проксирует `/api` на `http://localhost:3000` — бэкенд нужно запустить отдельно: `docker-compose up web` (без gateway, бэкенд снова на 3000).

## Бинарный формат

`branchy compile in.branchy -o out.branchyc` — в бинарник попадает уже разрешённая программа (include и ...include подставлены). Запуск: `branchy run out.branchyc`.
