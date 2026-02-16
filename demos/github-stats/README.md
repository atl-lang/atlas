# GitHub Stats Dashboard 📊

A real-world example of an Atlas application that fetches and displays GitHub user statistics using the GitHub API.

## What This Demonstrates

This demo showcases Atlas's full capabilities in a production-like scenario:

- **🌐 HTTP Client** - Making REST API calls to GitHub
- **📊 JSON** - Parsing and manipulating API responses
- **📅 DateTime** - Calculating time differences and formatting dates
- **🗂️ Collections** - Using HashMap to aggregate language statistics
- **💾 File I/O** - Caching API responses to avoid rate limits
- **🎨 String Manipulation** - Building beautiful CLI output with box-drawing
- **✅ Error Handling** - Result and Option types for robust error handling
- **📦 Module System** - Organized code across multiple modules

## Features

- ✨ Fetch user profile and repository statistics
- ⭐ Calculate total stars and forks across all repos
- 🏆 Find top repository by stars
- 💬 Aggregate programming languages and calculate percentages
- 📈 Calculate contribution streak from recent activity
- 🕒 Show time since last activity
- 💾 Smart caching (5 minutes) to avoid API rate limits
- 🎨 Beautiful CLI output with box-drawing characters

## Project Structure

```
github-stats/
├── atlas.toml          # Project configuration
├── main.atl            # Main entry point and orchestration
├── api.atl             # HTTP client for GitHub API
├── cache.atl           # File-based caching system
├── stats.atl           # Statistics calculation and aggregation
├── display.atl         # String formatting and display
├── data/               # Cache directory (auto-created)
└── README.md           # This file
```

## How to Run (For Beginners)

### Prerequisites

You need to have the Atlas runtime built. From the project root:

```bash
cd /path/to/atlas
cargo build -p atlas-runtime
```

### Running the Demo

**Option 1: Quick Run (Recommended)**

From the Atlas project root:

```bash
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/github-stats/main.atl
```

**Option 2: Custom Username**

1. Open `main.atl` in a text editor
2. Find this line near the bottom:
   ```atlas
   let username: string = "torvalds"; // Change this to any GitHub username
   ```
3. Change `"torvalds"` to any GitHub username you want to analyze
4. Run the demo using Option 1

### Example Output

```
GitHub Stats Dashboard
=====================

⏳ Fetching user profile for @torvalds...
⏳ Fetching repositories...
⏳ Fetching recent activity...
✅ Data fetched successfully (cached for 5 minutes)

┌─────────────────────────────────────────────────────┐
│           Linus Torvalds (@torvalds)                │
├─────────────────────────────────────────────────────┤
│ Public Repos:                                    25 │
│ Total Stars:                                 170000 │
│ Total Forks:                                  55000 │
│ Followers:                                   180000 │
├─────────────────────────────────────────────────────┤
│ Top Repo:                        linux (150000 ⭐) │
│ Top Language:                              C (85%) │
├─────────────────────────────────────────────────────┤
│ Last Active:                             2 days ago │
│ Contribution Streak:                         45 days│
└─────────────────────────────────────────────────────┘

💡 Tip: Data is cached for 5 minutes to avoid rate limits
```

## How It Works

### 1. **API Module** (`api.atl`)
Makes HTTP GET requests to GitHub's REST API with automatic caching:
- `/users/{username}` - User profile
- `/users/{username}/repos` - User repositories
- `/users/{username}/events/public` - Recent activity

### 2. **Cache Module** (`cache.atl`)
Implements a file-based cache with 5-minute TTL:
- Saves API responses as JSON files in `data/`
- Checks timestamps to determine cache validity
- Returns cached data if valid, otherwise fetches fresh data

### 3. **Stats Module** (`stats.atl`)
Calculates various statistics using Collections and Math:
- Aggregates total stars/forks across repositories
- Finds top repository by star count
- Uses HashMap to count repositories by language
- Calculates percentages and streaks

### 4. **Display Module** (`display.atl`)
Creates beautiful CLI output using String manipulation:
- Box-drawing characters for visual appeal
- String padding and alignment
- Number formatting

### 5. **Main Module** (`main.atl`)
Orchestrates the entire flow:
- Fetches data using Result types for error handling
- Calculates statistics from API responses
- Displays formatted output

## Error Handling

The demo uses Atlas's `Result<T, E>` and `Option<T>` types extensively:

```atlas
let result: Result<JsonValue, string> = fetchUser(username);

match result {
    Ok(data) -> // Use data
    Err(e) -> // Handle error
}
```

## Caching Behavior

- First run: Fetches data from GitHub API
- Subsequent runs (within 5 minutes): Uses cached data
- After 5 minutes: Automatically refreshes from API
- Cache files stored in `data/` directory

## Troubleshooting

**Error: "User not found"**
- Check that the username in `main.atl` is correct
- Make sure the user exists on GitHub

**Error: "Rate limit exceeded"**
- GitHub API has rate limits (60 requests/hour for unauthenticated)
- Wait a few minutes and try again
- The cache helps avoid hitting rate limits

**Error: "Network error"**
- Check your internet connection
- Verify you can access https://api.github.com

**Error: "Permission denied" (file I/O)**
- Make sure you're running with the `run_demo_allow_all` example
- The demo needs filesystem permissions for caching

## Customization Ideas

Try modifying the demo to:
- Add command-line argument support for username
- Display more statistics (recent commits, issue count, etc.)
- Add a visual bar chart for language percentages
- Compare multiple users side-by-side
- Export stats to JSON or CSV file
- Add colors to the output

## Learning Points

This demo is a great example of:
- **Modular design** - Separation of concerns across files
- **Error handling** - Using Result and Option types properly
- **Resource optimization** - Caching to reduce API calls
- **User experience** - Beautiful CLI output and helpful error messages
- **Real-world API integration** - Working with external REST APIs

## API Documentation

GitHub REST API: https://docs.github.com/en/rest

## Next Steps

After understanding this demo, check out the other demos to see different aspects of Atlas:
- `json-api-tester` - API testing with custom requests
- `weather-dashboard` - Multi-city weather tracking
- `web-crawler` - Link analysis and web scraping
- `rss-aggregator` - Feed parsing and aggregation
