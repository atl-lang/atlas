# Atlas Demos 🚀

Real-world example projects showcasing the full capabilities of the Atlas programming language.

## Available Demos

### 1. Feature Showcase (`feature-showcase/`)
**Quick reference for language features**

Demonstrates all stdlib features in isolated examples:
- Strings, Arrays, Math, Collections (HashMap, HashSet, Queue, Stack)
- Option/Result types with pattern matching
- Regex, DateTime, JSON
- File I/O with permissions
- Reflection API
- HTTP client

**Use case:** Quick reference and feature exploration

**How to run:**
```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/feature-showcase/main.atl
```

---

### 2. GitHub Stats Dashboard (`github-stats/`) ⭐
**Real-world API integration**

A production-quality CLI tool that fetches and displays GitHub user statistics.

**Showcases:**
- 🌐 HTTP client - REST API integration
- 📊 JSON parsing and manipulation
- 📅 DateTime - Calculate streaks and time differences
- 🗂️ Collections - HashMap for language aggregation
- 💾 File I/O - Response caching (5-minute TTL)
- 🎨 String formatting - Beautiful box-drawn output
- ✅ Result/Option types - Robust error handling

**Features:**
- Total repositories, stars, and forks
- Top repository and language statistics
- Contribution streak calculation
- Last activity tracking
- Smart caching to avoid rate limits

**How to run:**
```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/github-stats/main.atl
```

---

### 3. JSON API Testing Tool (`json-api-tester/`) 🧪
**HTTP API validation framework**

A comprehensive testing tool that reads test suites from JSON and validates API responses.

**Showcases:**
- 🌐 HTTP methods - GET, POST, PUT, DELETE, PATCH
- 📊 JSON - Test suite parsing and response validation
- ✅ Assertions - Status codes, headers, body content, JSON fields
- 💾 File I/O - Test suites and result persistence
- 📅 DateTime - Performance tracking
- 🗂️ Collections - Aggregating test results

**Features:**
- Multiple assertion types (status, headers, body, JSON)
- JSON test suite format
- Performance measurement
- Detailed error reporting
- Result export to JSON

**How to run:**
```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/json-api-tester/main.atl
```

---

### 4. Weather Dashboard CLI (`weather-dashboard/`) 🌤️
**Multi-source data aggregation**

A beautiful weather dashboard that tracks multiple cities with smart caching.

**Showcases:**
- 🌐 HTTP client - wttr.in weather API
- 📊 JSON - Parsing complex API responses
- 📅 DateTime - Cache TTL management
- 🧮 Math - Temperature unit conversions (C ↔ F)
- 💾 File I/O - Caching and configuration
- 🗂️ Collections - Multi-city tracking
- 🎨 String formatting - Beautiful CLI output with emojis

**Features:**
- Multiple city weather tracking
- Celsius/Fahrenheit support
- 30-minute smart caching
- Configurable city list
- No API key required
- Beautiful emoji weather icons

**How to run:**
```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/weather-dashboard/main.atl
```

---

### 5. Web Crawler & Link Analyzer (`web-crawler/`) 🕷️
**Graph traversal and data extraction**

A breadth-first web crawler that discovers pages and analyzes site structure.

**Showcases:**
- 🌐 HTTP client - Fetching web pages
- 🔍 Regex - Extracting links, emails, phone numbers
- 📊 Collections - Queue (BFS), HashSet (visited), HashMap (link graph)
- 💾 File I/O - Saving crawl results
- 📄 JSON export - Data for analysis
- 🎨 String manipulation - URL normalization
- 📅 DateTime - Crawl timestamps

**Features:**
- BFS (Breadth-First Search) algorithm
- Link graph construction
- URL normalization and validation
- Domain limiting
- Duplicate prevention with HashSet
- Statistics (average links, most linked pages)
- JSON export

**How to run:**
```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/web-crawler/main.atl
```

---

### 6. RSS Feed Aggregator (`rss-aggregator/`) 📰
**Content aggregation and parsing**

An RSS feed aggregator that combines multiple sources, deduplicates, and filters by date.

**Showcases:**
- 🌐 HTTP client - Fetching RSS feeds
- 🔍 Regex - Parsing XML/RSS without full XML parser
- 📅 DateTime - Date parsing and filtering
- 🗂️ Collections - HashSet for deduplication, HashMap for counting
- 💾 File I/O - Caching and configuration
- 📄 JSON export - Aggregated feed data
- 🎨 String manipulation - Text cleaning

**Features:**
- Multi-source RSS/Atom feed aggregation
- Deduplication by link
- Date range filtering
- Smart caching (15-minute TTL)
- Configurable feed sources
- Statistics by source
- JSON export

**How to run:**
```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/rss-aggregator/main.atl
```

---

## Quick Start

### Prerequisites

Build the Atlas runtime:

```bash
cd /path/to/atlas
cargo build -p atlas-runtime
```

### Running Any Demo

All demos use the same pattern:

```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/{demo-name}/main.atl
```

The `run_demo_allow_all` example grants necessary permissions for network and file I/O.

## What These Demos Teach

### Language Features
- **Type System** - Option<T>, Result<T,E>, pattern matching
- **Collections** - HashMap, HashSet, Queue, Stack
- **Strings** - Manipulation, formatting, parsing
- **Arrays** - Functional operations (map, filter, reduce)
- **Regex** - Pattern matching and extraction
- **DateTime** - Parsing, formatting, calculations
- **JSON** - Parsing, manipulation, export
- **Math** - Calculations and conversions

### Software Engineering
- **Modular design** - Separation of concerns
- **Error handling** - Result and Option types
- **Caching strategies** - TTL-based caching
- **API integration** - REST API consumption
- **Data aggregation** - Combining multiple sources
- **CLI UX** - Beautiful terminal output
- **Resource optimization** - Efficient data structures

### Algorithms & Data Structures
- **BFS** - Breadth-first search (web crawler)
- **HashSet** - Deduplication and visited tracking
- **HashMap** - Aggregation and counting
- **Queue** - FIFO processing
- **Parsing** - Regex-based data extraction

## Choosing a Demo

**Want to see API integration?** → GitHub Stats or Weather Dashboard

**Want to see testing patterns?** → JSON API Tester

**Want to see graph algorithms?** → Web Crawler

**Want to see data aggregation?** → RSS Feed Aggregator

**Want to see all features?** → Feature Showcase

## Demo Comparison

| Demo | HTTP | Regex | Collections | DateTime | File I/O | JSON | Complexity |
|------|------|-------|-------------|----------|----------|------|------------|
| Feature Showcase | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Low |
| GitHub Stats | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | Medium |
| API Tester | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Medium |
| Weather Dashboard | ✅ | ❌ | ✅ | ✅ | ✅ | ✅ | Medium |
| Web Crawler | ✅ | ✅ | ✅✅✅ | ✅ | ✅ | ✅ | High |
| RSS Aggregator | ✅ | ✅✅ | ✅✅ | ✅✅ | ✅ | ✅ | High |

**✅ = Uses feature | ✅✅ = Heavy usage**

## Project Structure

Each demo follows a consistent structure:

```
demo-name/
├── atlas.toml          # Project configuration
├── main.atl            # Main entry point
├── {module}.atl        # Functional modules
├── config/             # Configuration files
├── output/             # Generated output
└── README.md           # Detailed documentation
```

## Learning Path

**Beginner:** Start with Feature Showcase
↓
**Intermediate:** Try GitHub Stats or Weather Dashboard
↓
**Advanced:** Explore API Tester or Web Crawler
↓
**Expert:** RSS Aggregator (combines all concepts)

## Customization

All demos are designed to be easily customized:

- **GitHub Stats** - Change username in `main.atl`
- **API Tester** - Create custom test suites in `tests/`
- **Weather Dashboard** - Edit `config/cities.json`
- **Web Crawler** - Change start URL and max pages in `main.atl`
- **RSS Aggregator** - Add feeds to `feeds/feeds.json`

## Real-World Applications

These demos demonstrate patterns used in:

- **DevOps tools** - Monitoring, aggregation, API integration
- **CLI utilities** - Beautiful terminal interfaces
- **Data pipelines** - Fetching, parsing, transforming data
- **Testing frameworks** - API validation and testing
- **Content platforms** - RSS readers, news aggregators
- **Web scraping** - Data extraction and analysis

## Common Patterns

### Caching Pattern
```atlas
// Check cache first
let cached: Option<JsonValue> = getCache(key);
match cached {
    Some(data) -> return Ok(data),
    None -> {
        // Fetch fresh data
        let fresh: JsonValue = fetchData();
        // Cache it
        setCache(key, fresh);
        return Ok(fresh);
    }
}
```

### Error Handling Pattern
```atlas
let result: Result<Data, string> = riskyOperation();

match result {
    Ok(data) -> {
        // Process data
    },
    Err(e) -> {
        // Handle error
        print("Error: " + e);
    }
}
```

### Collection Pattern
```atlas
// Deduplication with HashSet
let seen: HashSet = setNew();
let unique: array = [];

for item in items {
    if !setHas(seen, item) {
        setAdd(seen, item);
        push(unique, item);
    }
}
```

## Contributing

These demos are designed to showcase Atlas capabilities. If you create additional demos:

1. Follow the established structure
2. Include comprehensive README
3. Add beginner-friendly instructions
4. Demonstrate multiple Atlas features
5. Include real-world use cases

## Troubleshooting

**"Permission denied" errors:**
- Make sure to use `run_demo_allow_all` example
- Demos need network and filesystem permissions

**"Module not found" errors:**
- Ensure you're running from the correct directory
- Check that all `.atl` files are present

**Network errors:**
- Check internet connection
- Some APIs may be temporarily unavailable
- Cached data will be used when available

## Atlas Language Features

For complete language documentation, see:
- `docs/specification/` - Language specification
- `STATUS.md` - Current implementation status
- Feature Showcase demo - Interactive feature reference

## Next Steps

After exploring these demos:

1. **Modify them** - Change parameters, add features
2. **Create your own** - Build custom Atlas applications
3. **Combine patterns** - Mix techniques from multiple demos
4. **Share your projects** - Help grow the Atlas ecosystem

## Support

For questions or issues:
- Read individual demo READMEs for detailed info
- Check `STATUS.md` for feature availability
- Review language specs in `docs/specification/`

---

**Happy coding with Atlas!** 🚀

These demos prove that Atlas is ready for real-world applications. From API integration to data processing, from testing frameworks to content aggregation - Atlas handles it all with elegance and power.
