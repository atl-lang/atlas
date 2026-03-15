# web

Official Atlas web framework — router, middleware, templates, HTMX.

Built on top of Atlas's `http.serve()` primitive. No central registry required.

## Install

```toml
[dependencies]
web = { git = "https://github.com/atl-pkg/web", tag = "v0.1.0" }
```

```sh
atlas install
```

## Usage

```atlas
import { Router, chain, logger, cors, htmx, render } from 'web';

const router = new_router()
    .get('/', fn(req, res) {
        return res.html(render('views/index.html', { title: 'Home' }));
    })
    .get('/users/:id', fn(req, res) {
        const id = req.params['id'];
        return res.json({ id });
    })
    .post('/counter', fn(req, res) {
        const count = req.json.unwrap()['count'] + 1;
        if htmx.is_htmx(req) {
            return htmx.partial(res, render('views/counter.html', { count }));
        }
        return res.html(render('views/page.html', { count }));
    });

const handler = chain([logger(), cors('*')], router.match_route);
http.serve(3000, handler);
```

## API

| Export | Description |
|--------|-------------|
| `new_router()` | Create a new Router |
| `Router.get/post/put/delete/patch` | Register route handlers |
| `chain(middleware, handler)` | Compose middleware |
| `logger()` | Request/response logger |
| `cors(origin)` | CORS headers |
| `json_body()` | Parse JSON request body |
| `rate_limit(max, window_ms)` | In-memory rate limiter |
| `basic_auth(validate)` | HTTP Basic Auth |
| `render(source, ctx)` | Render template string |
| `render_file(path, ctx)` | Render template file |
| `static_files(root)` | Serve static files |
| `htmx.is_htmx(req)` | Detect HTMX request |
| `htmx.partial(res, html)` | Return HTML partial |
| `htmx.redirect(res, url)` | HX-Redirect |
| `htmx.trigger(res, event)` | HX-Trigger |
| `htmx.oob(main, fragments)` | Out-of-band swaps |

## Part of the Atlas ecosystem

- Language: [atl-lang/atlas](https://github.com/atl-lang/atlas)
- Packages: [atl-pkg](https://github.com/atl-pkg)
