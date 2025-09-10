# Complete Drizzle ORM Setup Guide with Bun (Project Agnostic)

*Following Supabase's recommended approach with bun runtime*

## Step 1: Project Setup

```bash
# Create new project (or use existing)
mkdir my-drizzle-project
cd my-drizzle-project
bun init -y

# Install dependencies
bun add drizzle-orm postgres
bun add -d drizzle-kit @types/pg
```

## Step 2: Environment Setup

Create `.env` file:
```bash
# .env
DATABASE_URL="postgresql://username:password@localhost:5432/database_name"

# For Supabase (replace with your actual values)
# DATABASE_URL="postgresql://postgres.ibtbbmqbbwytbquxodrj:[YOUR-PASSWORD]@aws-1-us-east-1.pooler.supabase.com:5432/postgres"
```

Create `.env.example`:
```bash
# .env.example
DATABASE_URL="postgresql://username:password@localhost:5432/database_name"
```

## Step 3: Drizzle Configuration

Create `drizzle.config.ts`:
```typescript
// drizzle.config.ts
import type { Config } from 'drizzle-kit';

export default {
  schema: './drizzle/schema.ts',
  out: './drizzle/migrations',
  driver: 'pg',
  dbCredentials: {
    connectionString: process.env.DATABASE_URL!,
  },
  verbose: true,
  strict: true,
} satisfies Config;
```

## Step 4: Schema Definition (Supabase Style)

Create `drizzle/schema.ts`:
```typescript
// drizzle/schema.ts
import { pgTable, serial, text, varchar, timestamp, boolean, uuid } from "drizzle-orm/pg-core";

export const users = pgTable('users', {
  id: serial('id').primaryKey(),
  fullName: text('full_name'),
  phone: varchar('phone', { length: 256 }),
  email: varchar('email', { length: 256 }).unique().notNull(),
  isActive: boolean('is_active').default(true),
  createdAt: timestamp('created_at').defaultNow(),
  updatedAt: timestamp('updated_at').defaultNow(),
});

export const posts = pgTable('posts', {
  id: serial('id').primaryKey(),
  title: text('title').notNull(),
  content: text('content'),
  authorId: serial('author_id').references(() => users.id),
  published: boolean('published').default(false),
  createdAt: timestamp('created_at').defaultNow(),
  updatedAt: timestamp('updated_at').defaultNow(),
});

// Export types for TypeScript
export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
export type Post = typeof posts.$inferSelect;
export type NewPost = typeof posts.$inferInsert;
```

## Step 5: Database Connection

Create `drizzle/db.ts`:
```typescript
// drizzle/db.ts
import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import * as schema from './schema';

const connectionString = process.env.DATABASE_URL!;

// Disable prefetch as it is not supported for "Transaction" pool mode
const client = postgres(connectionString, { prepare: false });
export const db = drizzle(client, { schema });

// Export schema for use elsewhere
export { schema };
```

## Step 6: Package.json Scripts

Update `package.json`:
```json
{
  "scripts": {
    "db:generate": "drizzle-kit generate:pg",
    "db:migrate": "drizzle-kit migrate",
    "db:push": "drizzle-kit push:pg",
    "db:studio": "drizzle-kit studio",
    "db:seed": "bun run drizzle/seed.ts",
    "dev": "bun run index.ts"
  }
}
```

## Step 7: Generate and Run Migrations

```bash
# Generate migration files
bun run db:generate

# Apply migrations to database
bun run db:migrate

# Alternative: Push schema directly (good for development)
bun run db:push
```

## Step 8: Basic Usage

Create `index.ts`:
```typescript
// index.ts
import { db } from './drizzle/db';
import { users, posts } from './drizzle/schema';
import { eq, desc } from 'drizzle-orm';

// Create a new user
async function createUser() {
  const newUser = await db.insert(users).values({
    fullName: 'John Doe',
    email: 'john@example.com',
    phone: '+1234567890',
  }).returning();
  
  console.log('Created user:', newUser[0]);
  return newUser[0];
}

// Get all users
async function getAllUsers() {
  const allUsers = await db.select().from(users);
  console.log('All users:', allUsers);
  return allUsers;
}

// Get user by ID with posts
async function getUserWithPosts(userId: number) {
  const userWithPosts = await db.query.users.findFirst({
    where: eq(users.id, userId),
    with: {
      posts: {
        orderBy: [desc(posts.createdAt)],
      },
    },
  });
  
  console.log('User with posts:', userWithPosts);
  return userWithPosts;
}

// Create a post
async function createPost(authorId: number) {
  const newPost = await db.insert(posts).values({
    title: 'My First Post',
    content: 'This is the content of my first post.',
    authorId: authorId,
    published: true,
  }).returning();
  
  console.log('Created post:', newPost[0]);
  return newPost[0];
}

// Main function
async function main() {
  try {
    console.log('🚀 Starting Drizzle demo...');
    
    // Test database connection
    const users = await getAllUsers();
    console.log(`📊 Found ${users.length} users in database`);
    
    // Create a user if none exist
    if (users.length === 0) {
      const user = await createUser();
      await createPost(user.id);
    }
    
    console.log('✅ Demo completed successfully!');
  } catch (error) {
    console.error('❌ Error:', error);
  }
}

main();
```

## Step 9: Query Examples

Create `drizzle/queries.ts`:
```typescript
// drizzle/queries.ts
import { db } from './db';
import { users, posts, type NewUser, type NewPost } from './schema';
import { eq, like, and, desc, count } from 'drizzle-orm';

// User queries
export const userQueries = {
  // Create user
  create: async (userData: NewUser) => {
    const [user] = await db.insert(users).values(userData).returning();
    return user;
  },

  // Get all users
  getAll: async () => {
    return await db.select().from(users);
  },

  // Get user by ID
  getById: async (id: number) => {
    return await db.query.users.findFirst({
      where: eq(users.id, id),
    });
  },

  // Get user by email
  getByEmail: async (email: string) => {
    return await db.query.users.findFirst({
      where: eq(users.email, email),
    });
  },

  // Search users by name
  searchByName: async (name: string) => {
    return await db.select().from(users)
      .where(like(users.fullName, `%${name}%`));
  },

  // Update user
  update: async (id: number, userData: Partial<NewUser>) => {
    const [updated] = await db.update(users)
      .set({ ...userData, updatedAt: new Date() })
      .where(eq(users.id, id))
      .returning();
    return updated;
  },

  // Delete user
  delete: async (id: number) => {
    await db.delete(users).where(eq(users.id, id));
  },
};

// Post queries
export const postQueries = {
  // Create post
  create: async (postData: NewPost) => {
    const [post] = await db.insert(posts).values(postData).returning();
    return post;
  },

  // Get published posts
  getPublished: async (limit: number = 10) => {
    return await db.select().from(posts)
      .where(eq(posts.published, true))
      .orderBy(desc(posts.createdAt))
      .limit(limit);
  },

  // Get posts by author
  getByAuthor: async (authorId: number) => {
    return await db.select().from(posts)
      .where(eq(posts.authorId, authorId))
      .orderBy(desc(posts.createdAt));
  },

  // Get post with author info
  getWithAuthor: async (postId: number) => {
    return await db.select({
      post: posts,
      author: users,
    })
    .from(posts)
    .leftJoin(users, eq(posts.authorId, users.id))
    .where(eq(posts.id, postId));
  },

  // Count posts by user
  countByUser: async (userId: number) => {
    const result = await db.select({ count: count() })
      .from(posts)
      .where(eq(posts.authorId, userId));
    return result[0].count;
  },
};
```

## Step 10: Seed Data (Optional)

Create `drizzle/seed.ts`:
```typescript
// drizzle/seed.ts
import { db } from './db';
import { users, posts } from './schema';

async function seed() {
  console.log('🌱 Seeding database...');

  try {
    // Create sample users
    const sampleUsers = await db.insert(users).values([
      {
        fullName: 'Alice Johnson',
        email: 'alice@example.com',
        phone: '+1234567890',
      },
      {
        fullName: 'Bob Smith',
        email: 'bob@example.com',
        phone: '+0987654321',
      },
      {
        fullName: 'Charlie Brown',
        email: 'charlie@example.com',
        phone: '+1122334455',
      },
    ]).returning();

    console.log(`✅ Created ${sampleUsers.length} users`);

    // Create sample posts
    const samplePosts = await db.insert(posts).values([
      {
        title: 'Getting Started with Drizzle',
        content: 'Drizzle is an amazing ORM for TypeScript...',
        authorId: sampleUsers[0].id,
        published: true,
      },
      {
        title: 'Why I Love Bun',
        content: 'Bun is incredibly fast for JavaScript development...',
        authorId: sampleUsers[1].id,
        published: true,
      },
      {
        title: 'Draft Post',
        content: 'This is just a draft...',
        authorId: sampleUsers[2].id,
        published: false,
      },
    ]).returning();

    console.log(`✅ Created ${samplePosts.length} posts`);
    console.log('🎉 Seeding completed successfully!');

  } catch (error) {
    console.error('❌ Seeding failed:', error);
    throw error;
  }
}

seed();
```

## Step 11: Development Workflow

```bash
# 1. Start development
bun run dev

# 2. Make schema changes in drizzle/schema.ts

# 3. Generate migration
bun run db:generate

# 4. Apply migration
bun run db:migrate

# 5. Seed database (if needed)
bun run db:seed

# 6. View database in browser
bun run db:studio
```

## Step 12: Production Considerations

### Environment Variables for Production:
```bash
# Production .env
DATABASE_URL="postgresql://prod_user:secure_password@prod-db.example.com:5432/prod_database?sslmode=require"
NODE_ENV="production"
```

### Build Script:
```bash
# Add to package.json
{
  "scripts": {
    "build": "bun build index.ts --outdir ./dist",
    "start": "bun run dist/index.js",
    "db:migrate:prod": "DATABASE_URL=$PROD_DATABASE_URL bun run db:migrate"
  }
}
```

## Troubleshooting

### Common Issues:

1. **Connection Error**: Check DATABASE_URL format and database accessibility
2. **Migration Issues**: Ensure database exists and user has proper permissions
3. **Type Errors**: Run `bun run db:generate` after schema changes
4. **SSL Issues**: Add `?sslmode=require` to DATABASE_URL for production

### Supabase Specific:

1. **Row Level Security**: Remember to set up RLS policies in Supabase dashboard
2. **Connection Pooling**: Use transaction pooler URL for better performance
3. **Extensions**: Enable required PostgreSQL extensions in Supabase dashboard

This setup gives you a robust, type-safe database layer using Drizzle ORM with bun runtime, perfectly compatible with Supabase and other PostgreSQL providers!