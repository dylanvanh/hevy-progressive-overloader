# Hevy Progressive Overloader

This service automatically updates your Hevy workout routines with progressive overload suggestions using AI. It works through webhooks and has a backup system to make sure no workouts get missed.

## Setup

You'll need a few API keys first:

- Hevy API key
- Google Gemini API key (for the AI suggestions)
- A webhook token (for security)

Set them as environment variables:

```bash
export HEVY_API_KEY="your_hevy_api_key"
export WEBHOOK_TOKEN="your_webhook_token"
export GEMINI_API_KEY="your_gemini_api_key"
```

Then run:

```bash
cargo run
```

## How it Works

When you finish a workout in Hevy, it can send a webhook to this service. The service will:

1. Look at your workout and routine
2. Use AI to figure out what weights/sets you should do next
3. Update your routine with suggestions in the exercise notes

If webhooks don't work for some reason, there's also a backup that checks for new workouts every 15 minutes.

## API

- `POST /webhook` - The endpoint Hevy calls when workouts complete. Include `Authorization: Bearer <token>` in the headers.
