# RSS Feed Aggregator 📰

A powerful RSS feed aggregator that fetches multiple feeds, parses articles, deduplicates, filters by date, and exports to JSON.

## What This Demonstrates

- **🌐 HTTP Client** - Fetching RSS feeds from multiple sources
- **🔍 Regex** - Parsing XML/RSS without a full XML parser
- **📅 DateTime** - Parsing article dates, filtering by time range
- **🗂️ Collections** - HashSet for deduplication, HashMap for counting
- **💾 File I/O** - Caching feeds and reading configuration
- **📄 JSON** - Exporting aggregated feed data
- **🎨 String Manipulation** - Text cleaning and formatting
- **✅ Error Handling** - Robust feed processing with Result types

## Features

- 📡 **Multi-source aggregation** - Combine articles from many RSS feeds
- 🔄 **Deduplication** - Remove duplicate articles by link
- 📅 **Date filtering** - Show only recent articles (configurable)
- 💾 **Smart caching** - 15-minute cache to reduce network calls
- 📊 **Statistics** - Count articles by source
- 📝 **Configurable sources** - Edit feeds.json to customize
- 🎨 **Beautiful output** - Clean CLI display
- 📄 **JSON export** - Save aggregated feed for other tools

## Project Structure

```
rss-aggregator/
├── atlas.toml          # Project configuration
├── main.atl            # Main entry point and orchestration
├── parser.atl          # RSS/XML parsing with Regex
├── fetcher.atl         # HTTP fetching with caching
├── aggregator.atl      # Deduplication and merging
├── display.atl         # CLI output formatting
├── export.atl          # JSON export functionality
├── feeds/
│   └── feeds.json      # Feed sources configuration
├── cache/              # Cached feed data (auto-created)
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

**Quick Start:**

```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/rss-aggregator/main.atl
```

### Example Output

```
╔═══════════════════════════════════════════════════╗
║          RSS Feed Aggregator                      ║
╚═══════════════════════════════════════════════════╝

Loading feed sources...
Found 5 feed sources

📡 [1/5] Fetching: Hacker News
  🌐 Fetching fresh data...
  ✅ Hacker News (30 articles)

📡 [2/5] Fetching: The Verge
  📦 Using cached data
  ✅ The Verge (25 articles)

📡 [3/5] Fetching: TechCrunch
  🌐 Fetching fresh data...
  ✅ TechCrunch (20 articles)

📡 [4/5] Fetching: ArsTechnica
  🌐 Fetching fresh data...
  ✅ ArsTechnica (15 articles)

📡 [5/5] Fetching: Rust Blog
  🌐 Fetching fresh data...
  ✅ Rust Blog (10 articles)

✅ Fetched 5 of 5 feeds

Aggregating articles...
✅ Aggregation complete

═══════════════════════════════════════════════════
  Summary
═══════════════════════════════════════════════════
  Feeds Fetched:     5
  Total Articles:    100
  Unique Articles:   97
═══════════════════════════════════════════════════

Recent Articles (last 7 days):

─────────────────────────────────────────────────
1. New AI model achieves breakthrough performance
   Source: TechCrunch
   Date: Mon, 12 Feb 2024 14:30:00 GMT
   Link: https://techcrunch.com/2024/02/12/ai-breakthrough
   Google's latest AI model shows significant improvements...

─────────────────────────────────────────────────
2. Open source project hits major milestone
   Source: Hacker News
   Date: Mon, 12 Feb 2024 12:15:00 GMT
   Link: https://news.ycombinator.com/item?id=12345
   Popular open source framework releases version 2.0...

[... 8 more articles ...]

Exporting results...
📄 Feed exported to: ./output/aggregated-feed.json
📄 Summary exported to: ./output/feed-summary.json

💡 Tips:
  • Edit feeds/feeds.json to add/remove RSS sources
  • Feed data cached for 15 minutes
  • Results exported to output/aggregated-feed.json
```

## How It Works

### 1. **Parser Module** (`parser.atl`)
**RSS/XML parsing using Regex:**
- Extracts `<item>` or `<entry>` blocks
- Parses title, link, description, pubDate
- Handles CDATA sections
- Supports both RSS and Atom formats

**Key patterns:**
- Items: `<item[^>]*>(.*?)</item>`
- Tags: `<tag[^>]*>([^<]+)</tag>`
- CDATA: `<!\[CDATA\[(.*?)\]\]>`

### 2. **Fetcher Module** (`fetcher.atl`)
**HTTP fetching with smart caching:**
- Checks cache validity (15-minute TTL)
- Returns cached data if valid
- Fetches fresh data if expired
- Automatically caches responses

### 3. **Aggregator Module** (`aggregator.atl`)
**Article processing:**
- Merges articles from all feeds
- Deduplicates by link using HashSet
- Filters by date range (last N days)
- Counts articles by source

### 4. **Display Module** (`display.atl`)
**CLI formatting:**
- Shows progress for each feed
- Displays articles with metadata
- Truncates long text
- Shows summary statistics

### 5. **Export Module** (`export.atl`)
**JSON export:**
- Aggregated feed with all articles
- Summary statistics
- Timestamps for tracking

## Feed Configuration

Edit `feeds/feeds.json` to customize your sources:

```json
{
  "feeds": [
    {
      "name": "Your Feed Name",
      "url": "https://example.com/rss",
      "category": "Category"
    }
  ]
}
```

### Finding RSS Feeds

Most websites have RSS feeds:
- Look for RSS icon (🔶)
- Try `/rss`, `/feed`, `/atom.xml`
- Use browser extensions to find feeds
- Check website footer for RSS links

### Popular RSS Feeds

**Tech News:**
- Hacker News: https://news.ycombinator.com/rss
- The Verge: https://www.theverge.com/rss/index.xml
- TechCrunch: https://techcrunch.com/feed/
- Ars Technica: https://feeds.arstechnica.com/arstechnica/index

**Programming:**
- Rust Blog: https://blog.rust-lang.org/feed.xml
- GitHub Blog: https://github.blog/feed/
- Dev.to: https://dev.to/feed

**News:**
- NPR: https://feeds.npr.org/1001/rss.xml
- BBC: http://feeds.bbci.co.uk/news/rss.xml
- Reuters: https://www.reutersagency.com/feed/

## Customization

### Change Date Filter

In `main.atl`, find:
```atlas
let recentArticles: array = filterByDays(uniqueArticles, 7);
```

Change `7` to any number of days (e.g., `30` for last month).

### Change Display Count

In `main.atl`, find:
```atlas
let displayCount: number = 10;
```

Change `10` to show more/fewer articles.

### Change Cache Duration

In `fetcher.atl`, find:
```atlas
let CACHE_TTL: number = 900; // 15 minutes
```

Change to desired seconds (e.g., `3600` for 1 hour).

## Output Files

### aggregated-feed.json
All recent articles:
```json
{
  "timestamp": 1708123456,
  "articleCount": 97,
  "articles": [
    {
      "title": "Article Title",
      "link": "https://...",
      "description": "...",
      "pubDate": "Mon, 12 Feb 2024 14:30:00 GMT",
      "feedUrl": "https://...",
      "feedTitle": "Source Name"
    }
  ]
}
```

### feed-summary.json
Statistics:
```json
{
  "totalFeeds": 5,
  "successfulFeeds": 5,
  "totalArticles": 100,
  "uniqueArticles": 97,
  "recentArticles": 85
}
```

## Use Cases

- **News aggregation** - Stay updated from multiple sources
- **Research** - Monitor industry news and blogs
- **Content curation** - Collect articles for newsletters
- **Trend analysis** - Track topics across sources
- **Personal dashboard** - Custom news feed
- **Team updates** - Aggregate team blogs/releases

## Caching Behavior

- **First run**: Fetches all feeds (may take time)
- **Within 15 minutes**: Uses cached data (instant)
- **After 15 minutes**: Refreshes expired feeds
- **Cache location**: `cache/{feed-url}.json`

## Error Handling

The aggregator handles:
- Network errors (continues with other feeds)
- Invalid RSS format (skips malformed feeds)
- Missing dates (includes articles anyway)
- Duplicate articles (deduplicates by link)
- Feed fetch failures (reports error, continues)

## Troubleshooting

**Error: "Failed to read feeds config"**
- Check that `feeds/feeds.json` exists
- Verify JSON syntax is valid

**Error: "Failed to fetch feed"**
- Check internet connection
- Verify feed URL is correct and accessible
- Some feeds may be temporarily down

**No articles shown**
- Increase date filter range (e.g., 30 days)
- Check if feeds have recent articles
- Verify feeds are active

**Parsing errors**
- Some feeds use non-standard formats
- RSS/Atom variations may not parse correctly
- Check feed in browser to verify format

## Extending the Aggregator

Try adding:
- **Category filtering** - Filter by feed category
- **Keyword search** - Find articles by keyword
- **Sentiment analysis** - Analyze article tone
- **HTML export** - Generate readable HTML page
- **Email digest** - Send daily/weekly emails
- **Read/unread tracking** - Mark articles as read
- **Favorite articles** - Save interesting articles
- **Full-text extraction** - Fetch complete article content
- **OPML import/export** - Standard feed list format

## RSS/Atom Format Support

**Supported:**
- RSS 2.0 (most common)
- Atom 1.0 (partial)
- Basic item/entry fields

**Limitations:**
- No full XML parser (uses Regex)
- Complex nested elements may not parse
- Namespaced tags have limited support
- Media enclosures not extracted

For production use, consider a proper XML parser.

## Learning Points

This demo showcases:
- **Regex for parsing** - Extract data from semi-structured text
- **Collection patterns** - Using HashSet for deduplication
- **Date handling** - Parsing and filtering by datetime
- **Caching strategies** - Time-based cache invalidation
- **Error resilience** - Continue processing despite failures
- **Data aggregation** - Combining multiple data sources

## Real-World Applications

This pattern is used in:
- **RSS readers** - Feedly, Inoreader, NewsBlur
- **News aggregators** - Google News, Flipboard
- **Research tools** - Academic paper trackers
- **Monitoring services** - Brand mention tracking
- **Content platforms** - Aggregate blogs for platforms

## Privacy & Ethics

**RSS Best Practices:**
- ✅ Respect feed publisher's wishes
- ✅ Cache appropriately to reduce load
- ✅ Attribute sources properly
- ❌ Don't republish full content without permission
- ❌ Don't remove attribution or ads

## Next Steps

After exploring this demo, check out:
- `github-stats` - API integration patterns
- `web-crawler` - Content discovery
- `weather-dashboard` - Multi-source data aggregation
- `json-api-tester` - HTTP testing patterns

## Credits

RSS is an open standard for content syndication. This demo works with any standard RSS or Atom feed.
