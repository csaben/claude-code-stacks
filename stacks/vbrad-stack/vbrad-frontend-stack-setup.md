# VBRAD Stack Setup Guide
**Vite + Bun + React + Auth (Better-Auth) + Drizzle**

## Quick Start

```bash
# Create new Vite React project
bun create vite my-app --template react-ts
cd my-app

# Install VBRAD dependencies
bun add better-auth drizzle-orm postgres better-auth/react
bun add -D drizzle-kit @types/pg

# Initialize project
bun run dev
```

## 1. Project Structure

```
my-app/
├── db/
│   ├── schema.ts
│   └── index.ts
├── lib/
│   ├── auth.ts
│   └── auth-client.ts
├── src/
│   ├── components/
│   ├── App.tsx
│   └── main.tsx
├── .env.local
├── drizzle.config.ts
└── package.json
```

## 2. Environment Setup

### `.env.local`
```env
# Database
DATABASE_URL="postgresql://user:password@localhost:5432/myapp"

# Auth
BETTER_AUTH_SECRET="your-super-secret-key-min-32-characters-long"
BETTER_AUTH_URL="http://localhost:5173"

# OAuth (optional)
GITHUB_CLIENT_ID="your-github-client-id"
GITHUB_CLIENT_SECRET="your-github-client-secret"
```

### `.env.production`
```env
# Supabase Database
DATABASE_URL="postgresql://postgres:[password]@db.[project-ref].supabase.co:5432/postgres"

# Auth
BETTER_AUTH_SECRET="your-production-secret-different-from-dev"
BETTER_AUTH_URL="https://your-domain.com"

# OAuth
GITHUB_CLIENT_ID="your-production-github-client-id"
GITHUB_CLIENT_SECRET="your-production-github-client-secret"
```

## 3. Database Schema

### `db/schema.ts`
```typescript
import { pgTable, text, timestamp, boolean } from "drizzle-orm/pg-core"

export const user = pgTable("user", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
  email: text("email").notNull().unique(),
  emailVerified: boolean("emailVerified").notNull().default(false),
  image: text("image"),
  createdAt: timestamp("createdAt").notNull().defaultNow(),
  updatedAt: timestamp("updatedAt").notNull().defaultNow(),
})

export const session = pgTable("session", {
  id: text("id").primaryKey(),
  expiresAt: timestamp("expiresAt").notNull(),
  token: text("token").notNull().unique(),
  createdAt: timestamp("createdAt").notNull().defaultNow(),
  updatedAt: timestamp("updatedAt").notNull().defaultNow(),
  ipAddress: text("ipAddress"),
  userAgent: text("userAgent"),
  userId: text("userId").notNull().references(() => user.id, { onDelete: "cascade" }),
})

export const account = pgTable("account", {
  id: text("id").primaryKey(),
  accountId: text("accountId").notNull(),
  providerId: text("providerId").notNull(),
  userId: text("userId").notNull().references(() => user.id, { onDelete: "cascade" }),
  accessToken: text("accessToken"),
  refreshToken: text("refreshToken"),
  idToken: text("idToken"),
  accessTokenExpiresAt: timestamp("accessTokenExpiresAt"),
  refreshTokenExpiresAt: timestamp("refreshTokenExpiresAt"),
  scope: text("scope"),
  password: text("password"),
  createdAt: timestamp("createdAt").notNull().defaultNow(),
  updatedAt: timestamp("updatedAt").notNull().defaultNow(),
})

export const verification = pgTable("verification", {
  id: text("id").primaryKey(),
  identifier: text("identifier").notNull(),
  value: text("value").notNull(),
  expiresAt: timestamp("expiresAt").notNull(),
  createdAt: timestamp("createdAt").notNull().defaultNow(),
  updatedAt: timestamp("updatedAt").notNull().defaultNow(),
})
```

### `db/index.ts`
```typescript
import { drizzle } from 'drizzle-orm/postgres-js'
import postgres from 'postgres'
import * as schema from './schema'

const connectionString = process.env.DATABASE_URL!

if (!connectionString) {
  throw new Error('DATABASE_URL is required')
}

const client = postgres(connectionString, { 
  prepare: false,
  max: 1
})

export const db = drizzle(client, { schema })
export * from './schema'
```

## 4. Drizzle Configuration

### `drizzle.config.ts`
```typescript
import type { Config } from "drizzle-kit"

export default {
  schema: "./db/schema.ts",
  out: "./drizzle",
  driver: "pg",
  dbCredentials: {
    connectionString: process.env.DATABASE_URL!,
  },
  verbose: true,
  strict: true,
} satisfies Config
```

## 5. Better-Auth Setup

### `lib/auth.ts`
```typescript
import { betterAuth } from "better-auth"
import { drizzleAdapter } from "better-auth/adapters/drizzle"
import { db } from "../db"

export const auth = betterAuth({
  database: drizzleAdapter(db, {
    provider: "pg",
  }),
  emailAndPassword: {
    enabled: true,
    requireEmailVerification: false,
  },
  socialProviders: {
    github: {
      clientId: process.env.GITHUB_CLIENT_ID!,
      clientSecret: process.env.GITHUB_CLIENT_SECRET!,
    },
  },
  session: {
    expiresIn: 60 * 60 * 24 * 7, // 7 days
    updateAge: 60 * 60 * 24, // 1 day
  },
})

export type Session = typeof auth.$Infer.Session
export type User = typeof auth.$Infer.User
```

### `lib/auth-client.ts`
```typescript
import { createAuthClient } from "better-auth/react"

export const authClient = createAuthClient({
  baseURL: process.env.BETTER_AUTH_URL || "http://localhost:5173",
})

export const {
  signIn,
  signUp,
  signOut,
  useSession,
  getSession,
} = authClient
```

## 6. React Components

### `src/components/Auth.tsx`
```typescript
import { useSession, signIn, signOut } from "../lib/auth-client"

export function Auth() {
  const { data: session, isPending } = useSession()

  if (isPending) {
    return <div>Loading...</div>
  }

  if (!session) {
    return (
      <div className="auth-container">
        <h2>Sign In</h2>
        
        <button 
          onClick={() => signIn.email({ 
            email: "test@example.com", 
            password: "password123" 
          })}
        >
          Sign in with Email
        </button>
        
        <button onClick={() => signIn.social({ provider: "github" })}>
          Sign in with GitHub
        </button>
        
        <div>
          <h3>Or Sign Up</h3>
          <button 
            onClick={() => signUp.email({
              email: "newuser@example.com",
              password: "password123",
              name: "New User"
            })}
          >
            Sign Up
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="user-info">
      <h2>Welcome, {session.user.name}!</h2>
      <p>Email: {session.user.email}</p>
      <p>Session expires: {new Date(session.expiresAt).toLocaleString()}</p>
      <button onClick={() => signOut()}>Sign Out</button>
    </div>
  )
}
```

### `src/App.tsx`
```typescript
import { Auth } from './components/Auth'
import './App.css'

function App() {
  return (
    <div className="app">
      <h1>VBRAD Stack App</h1>
      <p>Vite + Bun + React + Auth + Drizzle</p>
      <Auth />
    </div>
  )
}

export default App
```

## 7. Package.json Scripts

```json
{
  "name": "vbrad-app",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "db:push": "bunx drizzle-kit push",
    "db:generate": "bunx drizzle-kit generate",
    "db:migrate": "bunx drizzle-kit migrate",
    "db:studio": "bunx drizzle-kit studio",
    "db:drop": "bunx drizzle-kit drop"
  }
}
```

## 8. Database Setup

### Local PostgreSQL
```bash
# Start PostgreSQL with Docker
docker run --name vbrad-postgres \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=myapp \
  -p 5432:5432 \
  -d postgres:15

# Push schema
bun run db:push

# View data
bun run db:studio
```

### Supabase Setup
```bash
# 1. Create project at https://supabase.com
# 2. Get DATABASE_URL from Settings > Database
# 3. Update .env.production
# 4. Push schema
bun run db:push
```

## 9. Development Workflow

```bash
# Start development server
bun run dev

# Make schema changes
# 1. Edit db/schema.ts
# 2. Push changes
bun run db:push

# View database
bun run db:studio

# Build for production
bun run build

# Preview production build
bun run preview
```

## 10. API Routes (Optional)

If you need API routes, create `src/api/auth/[...all].ts`:

```typescript
import { auth } from "../../../lib/auth"

export async function POST(request: Request) {
  return auth.handler(request)
}

export async function GET(request: Request) {
  return auth.handler(request)
}
```

## 11. Environment Variables Checklist

### Development
- [ ] `DATABASE_URL` - Local PostgreSQL connection
- [ ] `BETTER_AUTH_SECRET` - Random 32+ char string
- [ ] `BETTER_AUTH_URL` - http://localhost:5173

### Production
- [ ] `DATABASE_URL` - Supabase connection string
- [ ] `BETTER_AUTH_SECRET` - Different from dev, secure
- [ ] `BETTER_AUTH_URL` - Your production domain
- [ ] OAuth credentials (if using)

## 12. Deployment

### Vercel/Netlify
```bash
# Build command
bun run build

# Environment variables
# Add all production env vars to platform
```

### Docker
```dockerfile
FROM oven/bun:1

WORKDIR /app
COPY package.json bun.lockb ./
RUN bun install

COPY . .
RUN bun run build

EXPOSE 3000
CMD ["bun", "run", "preview", "--host", "0.0.0.0", "--port", "3000"]
```

The VBRAD stack gives you modern, fast, type-safe development with easy scaling from local to production!