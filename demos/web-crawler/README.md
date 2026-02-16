# Web Crawler & Link Analyzer 🕷️

A breadth-first web crawler that discovers pages, extracts links, and analyzes site structure using advanced data structures.

## What This Demonstrates

- **🌐 HTTP Client** - Fetching web pages
- **🔍 Regex** - Extracting links, emails, phone numbers from HTML
- **📊 Collections** - Queue (BFS), HashSet (visited tracking), HashMap (link graph)
- **💾 File I/O** - Saving crawl results
- **📄 JSON Export** - Exporting data for analysis
- **🎨 String Manipulation** - URL normalization and parsing
- **📅 DateTime** - Timestamps for crawl metadata
- **✅ Error Handling** - Robust crawling with Result types

## Features

- 🔍 **Breadth-First Search (BFS)** - Systematic page discovery
- 🗺️ **Link Graph** - Track relationships between pages
- 📊 **Statistics** - Average links, most linked pages
- 🎯 **Domain Limiting** - Only crawl same domain
- 📝 **Data Extraction** - Titles, descriptions, links
- 💾 **JSON Export** - Full results, summary, page list
- 🛡️ **URL Normalization** - Handle relative URLs, fragments
- ⚡ **Duplicate Prevention** - HashSet for visited tracking

## Project Structure

```
web-crawler/
├── atlas.toml          # Project configuration
├── main.atl            # Main entry point and display
├── crawler.atl         # BFS crawling logic with Queue
├── extractor.atl       # Regex-based content extraction
├── analyzer.atl        # Link analysis and statistics
├── urls.atl            # URL normalization and validation
├── export.atl          # JSON export functionality
├── config/
│   └── crawl-config.json  # Crawl configuration
├── output/             # Exported results (auto-created)
└── README.md           # This file
```

## How to Run (For Beginners)

### Prerequisites

Build the Atlas runtime:

```bash
cd /path/to/atlas
cargo build -p atlas-runtime
```

### Running the Demo

**Quick Start (crawls example.com):**

```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/web-crawler/main.atl
```

**Crawl a different site:**

1. Open `main.atl` in a text editor
2. Find these lines:
   ```atlas
   let startUrl: string = "https://example.com"; // Change this
   let maxPages: number = 10; // Limit crawl depth
   ```
3. Change the URL and max pages
4. Run the demo

### Example Output

```
╔═══════════════════════════════════════╗
║     Web Crawler & Link Analyzer      ║
╚═══════════════════════════════════════╝

Starting crawl...
URL: https://example.com
Max pages: 10

🔍 Crawling: https://example.com
🔍 Crawling: https://example.com/about
🔍 Crawling: https://example.com/contact
🔍 Crawling: https://example.com/services
🔍 Crawling: https://example.com/products

✅ Crawl completed!

═══════════════════════════════════════
  Crawl Summary
═══════════════════════════════════════
  Total Pages:    5
  Average Links:  8.4
  Domain:         example.com
═══════════════════════════════════════

Pages Discovered:
─────────────────────────────────────
1. Example Domain
   URL: https://example.com
   Links: 12

2. About Us
   URL: https://example.com/about
   Links: 8

3. Contact
   URL: https://example.com/contact
   Links: 6

4. Services
   URL: https://example.com/services
   Links: 10

5. Products
   URL: https://example.com/products
   Links: 9

Exporting results...
📄 Results exported to: ./output/crawl-results.json
📄 Summary exported to: ./output/crawl-summary.json
📄 Pages exported to: ./output/pages.json

✅ All data exported successfully!
```

## How It Works

### 1. **Crawler Module** (`crawler.atl`)
**Uses BFS (Breadth-First Search) algorithm:**
- Queue to store URLs to visit (FIFO)
- HashSet to track visited URLs (avoid duplicates)
- HashMap to store link graph (page → links)

**Algorithm:**
1. Add start URL to queue
2. Pop URL from queue
3. Check if already visited (skip if yes)
4. Fetch page HTML
5. Extract links from page
6. Add new links to queue
7. Mark current URL as visited
8. Repeat until queue empty or max pages reached

### 2. **Extractor Module** (`extractor.atl`)
**Uses Regex to extract:**
- Links: `href="([^"]+)"`
- Emails: `[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}`
- Phone numbers: `\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}`
- Page titles: `<title>([^<]+)</title>`
- Meta descriptions: `<meta name="description" content="([^"]+)"`

### 3. **URLs Module** (`urls.atl`)
**URL processing:**
- Validation: Check for valid HTTP/HTTPS
- Normalization: Remove fragments, trailing slashes
- Domain extraction: Get domain from URL
- Absolute conversion: Convert relative URLs to absolute
- Domain checking: Ensure same-domain crawling

### 4. **Analyzer Module** (`analyzer.atl`)
**Statistics:**
- Count incoming links per page
- Calculate average links per page
- Find most linked-to pages
- Generate crawl summary

### 5. **Export Module** (`export.atl`)
**JSON exports:**
- Full results with link graph
- Summary statistics
- Page list with metadata

## Collections Used

### Queue (BFS Traversal)
```atlas
let queue: Queue = queueNew();
queuePush(queue, url);
let next: Option<JsonValue> = queuePop(queue);
```

### HashSet (Visited Tracking)
```atlas
let visited: HashSet = setNew();
setAdd(visited, url);
if setHas(visited, url) { /* already visited */ }
```

### HashMap (Link Graph)
```atlas
let linkGraph: HashMap = mapNew();
mapSet(linkGraph, url, links);
let links: JsonValue = mapGet(linkGraph, url);
```

## Output Files

### crawl-results.json
Complete crawl data:
```json
{
  "timestamp": 1708123456,
  "data": {
    "pages": [...],
    "stats": {...}
  }
}
```

### crawl-summary.json
Summary statistics:
```json
{
  "totalPages": 10,
  "averageLinks": 8.4,
  "domain": "example.com",
  "startUrl": "https://example.com"
}
```

### pages.json
Page list:
```json
{
  "pages": [
    {
      "url": "https://example.com",
      "title": "Example Domain",
      "description": "...",
      "linkCount": 12
    }
  ],
  "count": 10
}
```

## Use Cases

- **SEO Analysis** - Analyze site structure and internal linking
- **Site Mapping** - Generate sitemap of website
- **Broken Link Detection** - Find dead links
- **Content Discovery** - Find all pages on a site
- **Data Mining** - Extract emails, phone numbers
- **Competitive Analysis** - Analyze competitor sites
- **Site Migration** - Map old site before migration

## Ethical Considerations

**⚠️ Important: Responsible Crawling**

- ✅ **Respect robots.txt** - Check site's crawling rules
- ✅ **Add delays** - Don't overload servers
- ✅ **Identify yourself** - Use meaningful User-Agent
- ✅ **Get permission** - For aggressive crawling
- ❌ **Don't scrape personal data** without consent
- ❌ **Don't bypass security** - Respect authentication
- ❌ **Don't overload servers** - Be a good citizen

## Extending the Crawler

Try adding:
- **Depth limiting** - Stop after N levels deep
- **robots.txt support** - Respect crawling rules
- **Rate limiting** - Delay between requests
- **Multi-threading** - Parallel crawling (when async available)
- **Content analysis** - Extract specific data
- **Broken link detection** - Check for 404s
- **Sitemap generation** - Create XML sitemap
- **Image extraction** - Find all images
- **Form detection** - Find forms on pages
- **JavaScript rendering** - Handle dynamic content

## Troubleshooting

**Error: "HTTP error"**
- Page returned non-200 status code
- Server might be blocking the crawler
- URL might be invalid

**Error: "Network error"**
- Check internet connection
- Firewall might be blocking requests
- Server might be down

**Few pages discovered**
- Site might have limited internal links
- Increase `maxPages` limit
- Check if robots.txt blocks crawling

**Crawler runs forever**
- Set a reasonable `maxPages` limit
- Large sites can have thousands of pages

## Learning Points

This demo showcases:
- **BFS algorithm** - Systematic graph traversal
- **Queue usage** - FIFO for level-order traversal
- **HashSet for deduplication** - O(1) lookup for visited check
- **HashMap for graphs** - Representing page relationships
- **Regex for parsing** - Extracting structured data from text
- **URL handling** - Normalization and validation
- **Error resilience** - Continue crawling despite failures

## Real-World Applications

This pattern is used in:
- **Search engines** - Google, Bing web crawlers
- **SEO tools** - Screaming Frog, Ahrefs
- **Archive services** - Wayback Machine
- **Security scanning** - Web vulnerability scanners
- **Monitoring services** - Uptime monitors

## Next Steps

After exploring this demo, check out:
- `github-stats` - API-driven data collection
- `json-api-tester` - HTTP request testing
- `rss-aggregator` - Feed parsing and aggregation

## Limitations

Current implementation:
- No JavaScript rendering (static HTML only)
- Simplified relative URL handling
- No robots.txt parsing
- No rate limiting
- Single-threaded (sequential crawling)

These would be addressed in a production crawler.

## Legal Disclaimer

Web crawling may be restricted by:
- Terms of Service of websites
- Computer Fraud and Abuse Act (CFAA)
- GDPR and privacy laws

Always:
- Read and respect the site's ToS
- Check robots.txt
- Get permission for commercial use
- Respect rate limits
- Don't scrape personal data

This demo is for educational purposes.
