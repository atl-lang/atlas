# Weather Dashboard CLI 🌤️

A beautiful multi-city weather dashboard that fetches live weather data and displays it in an elegant CLI format.

## What This Demonstrates

- **🌐 HTTP Client** - Fetching data from wttr.in weather API
- **📊 JSON** - Parsing complex API responses
- **📅 DateTime** - Timestamps for cache management
- **🧮 Math** - Temperature unit conversions (C ↔ F)
- **💾 File I/O** - Caching weather data and reading city configurations
- **🗂️ Collections** - Managing multiple cities
- **🎨 String Manipulation** - Beautiful CLI output with box-drawing
- **✅ Error Handling** - Graceful failures with Result types

## Features

- ✨ **Multi-city tracking** - Monitor weather in multiple locations
- 🌡️ **Temperature units** - Switch between Celsius and Fahrenheit
- 💾 **Smart caching** - 30-minute cache to minimize API calls
- 📍 **Configurable cities** - Edit cities.json to customize locations
- 🎨 **Beautiful output** - Box-drawn cards with emoji weather icons
- 🔄 **Auto-refresh** - Cache expires after 30 minutes
- 🆓 **No API key needed** - Uses free wttr.in service
- 🌈 **Weather emojis** - Visual weather conditions at a glance

## Project Structure

```
weather-dashboard/
├── atlas.toml          # Project configuration
├── main.atl            # Main entry point and orchestration
├── api.atl             # Weather API client
├── cache.atl           # File-based caching (30 min TTL)
├── weather.atl         # Weather data extraction and conversion
├── display.atl         # CLI display formatting
├── cities.atl          # City configuration management
├── config/
│   └── cities.json     # List of cities to track
├── cache/              # Cached weather data (auto-created)
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
cargo run -p atlas-runtime --example run_demo_allow_all -- demos/weather-dashboard/main.atl
```

### Example Output

```
╔═══════════════════════════════════════════════════════╗
║           Weather Dashboard - Live Data              ║
╚═══════════════════════════════════════════════════════╝

⏳ Fetching weather for San Francisco...
┌────────────────────────────────────────────────┐
│  San Francisco
├────────────────────────────────────────────────┤
│  18°C  ☁️  Partly cloudy
│  Feels like 17°C
├────────────────────────────────────────────────┤
│  Humidity: 65%
│  Wind: 15 km/h W
└────────────────────────────────────────────────┘

⏳ Fetching weather for New York...
┌────────────────────────────────────────────────┐
│  New York
├────────────────────────────────────────────────┤
│  12°C  ☀️  Clear
│  Feels like 10°C
├────────────────────────────────────────────────┤
│  Humidity: 45%
│  Wind: 8 km/h NE
└────────────────────────────────────────────────┘

⏳ Fetching weather for London...
┌────────────────────────────────────────────────┐
│  London
├────────────────────────────────────────────────┤
│  8°C  🌧️  Light rain
│  Feels like 6°C
├────────────────────────────────────────────────┤
│  Humidity: 85%
│  Wind: 20 km/h SW
└────────────────────────────────────────────────┘

───────────────────────────────────────────────────────
Summary: 8/8 cities loaded successfully
Temperature unit: Celsius
Cache: Data cached for 30 minutes
───────────────────────────────────────────────────────

💡 Tips:
  • Edit config/cities.json to add/remove cities
  • Weather data is cached for 30 minutes
  • Uses wttr.in public API (no API key needed)
```

## Customization

### Adding/Removing Cities

Edit `config/cities.json`:

```json
{
  "cities": [
    "San Francisco",
    "New York",
    "London",
    "Your City Here"
  ]
}
```

### Changing Temperature Units

Open `main.atl` and find this line:

```atlas
let useMetric: bool = true; // Change to false for Fahrenheit
```

Change to:

```atlas
let useMetric: bool = false; // Uses Fahrenheit
```

## How It Works

### 1. **API Module** (`api.atl`)
- Fetches weather data from wttr.in in JSON format
- URL: `https://wttr.in/{city}?format=j1`
- Automatically caches responses for 30 minutes
- No API key required!

### 2. **Cache Module** (`cache.atl`)
- Stores weather data in `cache/` directory
- Each city has its own cache file
- 30-minute TTL (time-to-live)
- Automatically checks cache validity

### 3. **Weather Module** (`weather.atl`)
- Extracts temperature, description, humidity, wind data
- Converts between Celsius and Fahrenheit
- Selects emoji based on weather conditions
- Provides "feels like" temperature

### 4. **Display Module** (`display.atl`)
- Creates beautiful box-drawn cards
- Formats weather information clearly
- Shows summary statistics
- Provides helpful tips

### 5. **Cities Module** (`cities.atl`)
- Loads city list from JSON configuration
- Falls back to default cities if config fails
- Easy to extend with new cities

### 6. **Main Module** (`main.atl`)
- Orchestrates the entire flow
- Fetches weather for each city
- Handles errors gracefully
- Displays results

## Weather Emojis

The dashboard uses emojis to visualize weather conditions:

- ☀️ Sunny / Clear
- ☁️ Cloudy
- 🌧️ Rainy
- ❄️ Snowy
- ⛈️ Thunderstorm
- 🌫️ Foggy / Misty
- 💨 Windy
- 🌤️ Default

## Caching Behavior

- **First run**: Fetches fresh data from API
- **Within 30 minutes**: Uses cached data (instant)
- **After 30 minutes**: Automatically fetches fresh data
- **Cache location**: `cache/{city-name}.json`

## API Information

**wttr.in**
- Free weather API
- No registration required
- No API key needed
- Rate limit: Reasonable for personal use
- Format: `?format=j1` returns JSON
- Website: https://wttr.in/:help

## Error Handling

The dashboard gracefully handles:
- Network errors (no internet connection)
- Invalid city names (shows error card)
- API failures (continues with other cities)
- Missing config file (uses default cities)
- Cache corruption (fetches fresh data)

## Troubleshooting

**Error: "Failed to read cities config"**
- Make sure `config/cities.json` exists
- Check that JSON syntax is valid

**Error: "Weather API error: 404"**
- City name not found
- Check spelling of city name
- Try using the full city name (e.g., "Los Angeles, CA")

**Error: "HTTP request failed"**
- Check your internet connection
- Verify you can access https://wttr.in

**Slow loading**
- First run fetches fresh data (may take a few seconds per city)
- Subsequent runs use cache (instant)
- Cache expires after 30 minutes

**Want faster updates?**
- Edit `cache.atl` and change `CACHE_TTL` value
- Default: 1800 seconds (30 minutes)
- Example: 300 seconds (5 minutes)

## Extending the Dashboard

Try adding:
- **7-day forecast** - wttr.in provides forecast data
- **Historical comparisons** - Compare with yesterday
- **Weather alerts** - Highlight severe weather
- **Charts/graphs** - ASCII art weather trends
- **Sunrise/sunset times** - Available in API data
- **Multiple locations per city** - Track neighborhoods
- **Export to file** - Save weather history
- **Command-line args** - Specify city from CLI
- **Favorite cities** - Star/pin certain cities
- **Notification system** - Alert on weather changes

## Learning Points

This demo showcases:
- **API integration** - Working with real-world weather APIs
- **Caching strategies** - Optimizing API usage with TTL cache
- **Unit conversions** - Mathematical operations (C ↔ F)
- **Configuration management** - External JSON config files
- **Error resilience** - Continue operation when individual cities fail
- **User experience** - Beautiful CLI output with helpful feedback
- **Data extraction** - Parsing nested JSON structures

## Real-World Applications

This pattern is used in:
- **DevOps dashboards** - Monitoring services across regions
- **Stock tickers** - Multi-symbol price tracking
- **Server monitoring** - Health checks for multiple servers
- **Social media feeds** - Aggregating updates from multiple sources

## Next Steps

After exploring this demo, check out:
- `github-stats` - Single-entity deep dive
- `json-api-tester` - API validation and testing
- `web-crawler` - Link discovery and analysis
- `rss-aggregator` - Multi-source content aggregation

## Credits

Weather data provided by **wttr.in** - a console-oriented weather forecast service.
