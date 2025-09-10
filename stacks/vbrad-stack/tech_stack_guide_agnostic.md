# Tech Stack Preferences

## Package Manager
**Always use Bun** - never npm, yarn, or pnpm
```bash
bun install    # not npm install
bun add pkg    # not npm install pkg  
bun run dev    # not npm run dev
```

## Stack
- **Frontend**: React + Vite + Drizzle ORM
- **Backend**: Bun runtime
- **Database**: PostgreSQL + Drizzle ORM