not mandatory but good reference:
```package.json

  "scripts": {
    "dev": "concurrently \"bun run dev:frontend\" \"bun run dev:backend\"",
    "dev:frontend": "vite",
    "dev:backend": "bun --watch server.ts",
    "build": "tsc -b && vite build",
    "lint": "eslint .",
    "preview": "vite preview",
    "db:push": "drizzle-kit push",
    "db:generate": "drizzle-kit generate",
    "db:studio": "drizzle-kit studio"
  },
```