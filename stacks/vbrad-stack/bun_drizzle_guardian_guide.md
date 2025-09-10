# Guardian Project Drizzle Setup with Bun

*Complete setup for your personal knowledge TikTok app with vector embeddings*

## Step 1: Project Setup

```bash
cd api_server
bun add drizzle-orm postgres pgvector
bun add -d drizzle-kit @types/pg
```

## Step 2: Environment Configuration

Ensure your `.env` has:
```bash
# .env
DATABASE_URL="postgresql://username:password@localhost:5432/guardian_db"

# For Supabase (with pgvector enabled)
# DATABASE_URL="postgresql://postgres:[PASSWORD]@db.[PROJECT-REF].supabase.co:5432/postgres"
```

## Step 3: Drizzle Configuration

Create `drizzle.config.ts`:
```typescript
// api_server/drizzle.config.ts
import type { Config } from 'drizzle-kit';

export default {
  schema: './src/db/schema/index.ts',
  out: './src/db/migrations',
  driver: 'pg',
  dbCredentials: {
    connectionString: process.env.DATABASE_URL!,
  },
  verbose: true,
  strict: true,
} satisfies Config;
```

## Step 4: Database Connection

Create `src/db/connection.ts`:
```typescript
// src/db/connection.ts
import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import * as schema from './schema';

const connectionString = process.env.DATABASE_URL!;

// Disable prefetch as it is not supported for "Transaction" pool mode
const client = postgres(connectionString, { prepare: false });
export const db = drizzle(client, { schema });

// Export for convenience
export { schema };
export * from './schema';
```

## Step 5: Guardian Schema Definition

### Base Schema (`src/db/schema/users.ts`)
```typescript
// src/db/schema/users.ts
import { pgTable, uuid, text, timestamp, jsonb } from 'drizzle-orm/pg-core';

export const users = pgTable('users', {
  id: uuid('id').defaultRandom().primaryKey(),
  email: text('email').unique().notNull(),
  displayName: text('display_name'),
  avatarUrl: text('avatar_url'),
  preferences: jsonb('preferences').default({}),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow(),
});

export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
```

### Uploads Schema (`src/db/schema/uploads.ts`)
```typescript
// src/db/schema/uploads.ts
import { pgTable, uuid, text, timestamp, jsonb, boolean, bigint, index } from 'drizzle-orm/pg-core';
import { users } from './users';

export const uploads = pgTable('uploads', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  name: text('name').notNull(),
  originalFilename: text('original_filename'),
  contentType: text('content_type', { 
    enum: ['audio', 'video', 'text', 'external_scrape'] 
  }).notNull(),
  fileSizeBytes: bigint('file_size_bytes', { mode: 'number' }),
  storageUrl: text('storage_url'),
  transcript: text('transcript'),
  processingStatus: text('processing_status', {
    enum: ['pending', 'processing', 'completed', 'failed']
  }).default('pending'),
  transcriptionStatus: text('transcription_status', {
    enum: ['pending', 'processing', 'completed', 'failed']
  }).default('pending'),
  embeddingStatus: text('embedding_status', {
    enum: ['pending', 'processing', 'completed', 'failed']
  }).default('pending'),
  sourceUrl: text('source_url'),
  sourceType: text('source_type'),
  sourceMetadata: jsonb('source_metadata').default({}),
  isPrivate: boolean('is_private').default(false),
  isFavorite: boolean('is_favorite').default(false),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow(),
}, (table) => ({
  userIdIdx: index('uploads_user_id_idx').on(table.userId),
  contentTypeIdx: index('uploads_content_type_idx').on(table.contentType),
  processingStatusIdx: index('uploads_processing_status_idx').on(table.processingStatus),
  createdAtIdx: index('uploads_created_at_idx').on(table.createdAt),
}));

export type Upload = typeof uploads.$inferSelect;
export type NewUpload = typeof uploads.$inferInsert;
```

### Chunks with Vector Embeddings (`src/db/schema/chunks.ts`)
```typescript
// src/db/schema/chunks.ts
import { pgTable, uuid, text, timestamp, integer, index, vector } from 'drizzle-orm/pg-core';
import { uploads } from './uploads';

export const chunks = pgTable('chunks', {
  id: uuid('id').defaultRandom().primaryKey(),
  uploadId: uuid('upload_id').notNull().references(() => uploads.id, { onDelete: 'cascade' }),
  content: text('content').notNull(),
  embedding: vector('embedding', { dimensions: 1536 }),
  chunkIndex: integer('chunk_index').notNull(),
  totalChunks: integer('total_chunks').notNull(),
  startOffset: integer('start_offset'),
  endOffset: integer('end_offset'),
  summary: text('summary'),
  summaryInContext: text('summary_in_context'),
  keywords: text('keywords').array(),
  prevChunkSummary: text('prev_chunk_summary'),
  nextChunkSummary: text('next_chunk_summary'),
  chunkingStrategy: text('chunking_strategy').default('semantic'),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
}, (table) => ({
  uploadIdIdx: index('chunks_upload_id_idx').on(table.uploadId),
  embeddingIdx: index('chunks_embedding_idx').using('ivfflat', table.embedding.op('vector_cosine_ops')),
  uniqueChunkIdx: index('chunks_upload_chunk_unique_idx').on(table.uploadId, table.chunkIndex),
}));

export type Chunk = typeof chunks.$inferSelect;
export type NewChunk = typeof chunks.$inferInsert;
```

### Threads for Custom Feeds (`src/db/schema/threads.ts`)
```typescript
// src/db/schema/threads.ts
import { pgTable, uuid, text, timestamp, jsonb, boolean, index } from 'drizzle-orm/pg-core';
import { users } from './users';

export const threads = pgTable('threads', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  name: text('name').notNull(),
  description: text('description'),
  threadType: text('thread_type', {
    enum: ['custom_feed', 'conversation', 'search_session']
  }).default('custom_feed'),
  config: jsonb('config').default({}),
  isActive: boolean('is_active').default(true),
  lastAccessed: timestamp('last_accessed', { withTimezone: true }).defaultNow(),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow(),
}, (table) => ({
  userIdIdx: index('threads_user_id_idx').on(table.userId),
  threadTypeIdx: index('threads_type_idx').on(table.threadType),
  lastAccessedIdx: index('threads_last_accessed_idx').on(table.lastAccessed),
}));

export type Thread = typeof threads.$inferSelect;
export type NewThread = typeof threads.$inferInsert;
```

### Albums (`src/db/schema/albums.ts`)
```typescript
// src/db/schema/albums.ts
import { pgTable, uuid, text, timestamp, boolean, index, primaryKey } from 'drizzle-orm/pg-core';
import { users } from './users';
import { uploads } from './uploads';

export const albums = pgTable('albums', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  name: text('name').notNull(),
  description: text('description'),
  isPrivate: boolean('is_private').default(false),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
  updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow(),
}, (table) => ({
  userIdIdx: index('albums_user_id_idx').on(table.userId),
}));

export const albumUploads = pgTable('album_uploads', {
  albumId: uuid('album_id').notNull().references(() => albums.id, { onDelete: 'cascade' }),
  uploadId: uuid('upload_id').notNull().references(() => uploads.id, { onDelete: 'cascade' }),
  addedAt: timestamp('added_at', { withTimezone: true }).defaultNow(),
}, (table) => ({
  pk: primaryKey({ columns: [table.albumId, table.uploadId] }),
}));

export type Album = typeof albums.$inferSelect;
export type NewAlbum = typeof albums.$inferInsert;
export type AlbumUpload = typeof albumUploads.$inferSelect;
```

### Thread Interactions (`src/db/schema/interactions.ts`)
```typescript
// src/db/schema/interactions.ts
import { pgTable, uuid, text, timestamp, integer, index } from 'drizzle-orm/pg-core';
import { users } from './users';
import { threads } from './threads';
import { uploads } from './uploads';

export const threadInteractions = pgTable('thread_interactions', {
  id: uuid('id').defaultRandom().primaryKey(),
  threadId: uuid('thread_id').notNull().references(() => threads.id, { onDelete: 'cascade' }),
  uploadId: uuid('upload_id').notNull().references(() => uploads.id, { onDelete: 'cascade' }),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  interactionType: text('interaction_type', {
    enum: ['view', 'like', 'bookmark', 'skip', 'share']
  }).notNull(),
  durationSeconds: integer('duration_seconds'),
  positionInFeed: integer('position_in_feed'),
  algorithmUsed: text('algorithm_used'),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
}, (table) => ({
  threadIdIdx: index('thread_interactions_thread_id_idx').on(table.threadId),
  userIdIdx: index('thread_interactions_user_id_idx').on(table.userId),
  createdAtIdx: index('thread_interactions_created_at_idx').on(table.createdAt),
}));

export type ThreadInteraction = typeof threadInteractions.$inferSelect;
export type NewThreadInteraction = typeof threadInteractions.$inferInsert;
```

### Search Sessions (`src/db/schema/searchSessions.ts`)
```typescript
// src/db/schema/searchSessions.ts
import { pgTable, uuid, text, timestamp, jsonb, integer, real, index, vector } from 'drizzle-orm/pg-core';
import { users } from './users';
import { threads } from './threads';

export const searchSessions = pgTable('search_sessions', {
  id: uuid('id').defaultRandom().primaryKey(),
  userId: uuid('user_id').notNull().references(() => users.id, { onDelete: 'cascade' }),
  threadId: uuid('thread_id').references(() => threads.id, { onDelete: 'set null' }),
  query: text('query').notNull(),
  queryEmbedding: vector('query_embedding', { dimensions: 1536 }),
  filters: jsonb('filters').default({}),
  maxResults: integer('max_results').default(20),
  similarityThreshold: real('similarity_threshold').default(0.7),
  results: jsonb('results'),
  resultCount: integer('result_count'),
  createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
}, (table) => ({
  userIdIdx: index('search_sessions_user_id_idx').on(table.userId),
  queryEmbeddingIdx: index('search_sessions_query_embedding_idx').using('ivfflat', table.queryEmbedding.op('vector_cosine_ops')),
}));

export type SearchSession = typeof searchSessions.$inferSelect;
export type NewSearchSession = typeof searchSessions.$inferInsert;
```

### Schema Index (`src/db/schema/index.ts`)
```typescript
// src/db/schema/index.ts
export * from './users';
export * from './uploads';
export * from './chunks';
export * from './threads';
export * from './albums';
export * from './interactions';
export * from './searchSessions';
```

## Step 6: Package.json Scripts

Update `package.json`:
```json
{
  "scripts": {
    "dev": "bun --watch src/server.ts",
    "db:generate": "drizzle-kit generate:pg",
    "db:migrate": "drizzle-kit migrate",
    "db:push": "drizzle-kit push:pg",
    "db:studio": "drizzle-kit studio",
    "db:seed": "bun run src/db/seed.ts",
    "db:reset": "bun run src/db/reset.ts"
  }
}
```

## Step 7: Enable Extensions and Migrate

Create `src/db/extensions.sql`:
```sql
-- src/db/extensions.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS vector;
```

Run setup:
```bash
# Enable extensions first
psql $DATABASE_URL -f src/db/extensions.sql

# Generate migrations
bun run db:generate

# Apply migrations
bun run db:migrate
```

## Step 8: Guardian-Specific Queries

### Upload Queries (`src/db/queries/uploads.ts`)
```typescript
// src/db/queries/uploads.ts
import { db } from '../connection';
import { uploads, chunks, users, type NewUpload } from '../schema';
import { eq, and, desc, sql, inArray } from 'drizzle-orm';

export const uploadQueries = {
  create: async (data: NewUpload) => {
    const [upload] = await db.insert(uploads).values(data).returning();
    return upload;
  },

  getById: async (id: string) => {
    return await db.query.uploads.findFirst({
      where: eq(uploads.id, id),
      with: {
        chunks: {
          orderBy: [chunks.chunkIndex],
        },
      },
    });
  },

  getUserUploads: async (userId: string, options: {
    limit?: number;
    offset?: number;
    contentType?: string;
    processingStatus?: string;
  } = {}) => {
    const { limit = 20, offset = 0, contentType, processingStatus } = options;
    
    let whereConditions = [eq(uploads.userId, userId)];
    
    if (contentType) {
      whereConditions.push(eq(uploads.contentType, contentType));
    }
    
    if (processingStatus) {
      whereConditions.push(eq(uploads.processingStatus, processingStatus));
    }

    return await db.select()
      .from(uploads)
      .where(and(...whereConditions))
      .orderBy(desc(uploads.createdAt))
      .limit(limit)
      .offset(offset);
  },

  updateProcessingStatus: async (id: string, status: string) => {
    const [updated] = await db
      .update(uploads)
      .set({ 
        processingStatus: status,
        updatedAt: new Date(),
      })
      .where(eq(uploads.id, id))
      .returning();
    return updated;
  },

  getProcessingQueue: async (status: string = 'pending') => {
    return await db.select()
      .from(uploads)
      .where(eq(uploads.processingStatus, status))
      .orderBy(uploads.createdAt);
  },
};
```

### Vector Search Queries (`src/db/queries/search.ts`)
```typescript
// src/db/queries/search.ts
import { db } from '../connection';
import { chunks, uploads, searchSessions, type NewSearchSession } from '../schema';
import { eq, and, sql, desc } from 'drizzle-orm';

export const searchQueries = {
  vectorSimilarity: async (
    queryEmbedding: number[],
    userId: string,
    options: {
      limit?: number;
      similarityThreshold?: number;
      contentTypes?: string[];
      excludeUploads?: string[];
    } = {}
  ) => {
    const { 
      limit = 20, 
      similarityThreshold = 0.7, 
      contentTypes,
      excludeUploads 
    } = options;

    let whereConditions = [
      eq(uploads.userId, userId),
      sql`${chunks.embedding} <=> ${queryEmbedding} < ${similarityThreshold}`
    ];

    if (contentTypes?.length) {
      whereConditions.push(sql`${uploads.contentType} = ANY(${contentTypes})`);
    }

    if (excludeUploads?.length) {
      whereConditions.push(sql`${uploads.id} != ALL(${excludeUploads})`);
    }

    return await db
      .select({
        chunk: chunks,
        upload: uploads,
        similarity: sql<number>`1 - (${chunks.embedding} <=> ${queryEmbedding})`.as('similarity')
      })
      .from(chunks)
      .innerJoin(uploads, eq(chunks.uploadId, uploads.id))
      .where(and(...whereConditions))
      .orderBy(sql`${chunks.embedding} <=> ${queryEmbedding}`)
      .limit(limit);
  },

  saveSession: async (sessionData: NewSearchSession) => {
    const [session] = await db.insert(searchSessions)
      .values(sessionData)
      .returning();
    return session;
  },

  getRecentSearches: async (userId: string, limit: number = 10) => {
    return await db.select()
      .from(searchSessions)
      .where(eq(searchSessions.userId, userId))
      .orderBy(desc(searchSessions.createdAt))
      .limit(limit);
  },
};
```

### Thread/Feed Queries (`src/db/queries/threads.ts`)
```typescript
// src/db/queries/threads.ts
import { db } from '../connection';
import { threads, threadInteractions, uploads, chunks, type NewThread, type NewThreadInteraction } from '../schema';
import { eq, and, desc, sql, inArray } from 'drizzle-orm';

export const threadQueries = {
  create: async (data: NewThread) => {
    const [thread] = await db.insert(threads).values(data).returning();
    return thread;
  },

  getUserThreads: async (userId: string) => {
    return await db.select()
      .from(threads)
      .where(eq(threads.userId, userId))
      .orderBy(desc(threads.lastAccessed));
  },

  recordInteraction: async (data: NewThreadInteraction) => {
    const [interaction] = await db.insert(threadInteractions)
      .values(data)
      .returning();

    // Update thread last accessed
    await db.update(threads)
      .set({ lastAccessed: new Date() })
      .where(eq(threads.id, data.threadId));

    return interaction;
  },

  getThreadContext: async (threadId: string, limit: number = 10) => {
    return await db
      .select({
        interaction: threadInteractions,
        upload: uploads,
        chunk: chunks,
      })
      .from(threadInteractions)
      .innerJoin(uploads, eq(threadInteractions.uploadId, uploads.id))
      .leftJoin(chunks, eq(uploads.id, chunks.uploadId))
      .where(eq(threadInteractions.threadId, threadId))
      .orderBy(desc(threadInteractions.createdAt))
      .limit(limit);
  },

  generateFeed: async (
    userId: string,
    threadId?: string,
    options: {
      algorithm?: 'random' | 'recent' | 'similarity';
      limit?: number;
      excludeRecent?: boolean;
    } = {}
  ) => {
    const { algorithm = 'recent', limit = 20, excludeRecent = true } = options;

    // Get recently viewed uploads to exclude
    let excludeUploads: string[] = [];
    if (excludeRecent && threadId) {
      const recentInteractions = await db
        .select({ uploadId: threadInteractions.uploadId })
        .from(threadInteractions)
        .where(eq(threadInteractions.threadId, threadId))
        .orderBy(desc(threadInteractions.createdAt))
        .limit(10);
      
      excludeUploads = recentInteractions.map(i => i.uploadId);
    }

    let baseQuery = db
      .select({
        upload: uploads,
        chunk: chunks,
      })
      .from(uploads)
      .leftJoin(chunks, eq(uploads.id, chunks.uploadId))
      .where(and(
        eq(uploads.userId, userId),
        eq(uploads.processingStatus, 'completed'),
        excludeUploads.length > 0 ? 
          sql`${uploads.id} != ALL(${excludeUploads})` : 
          sql`true`
      ))
      .limit(limit);

    switch (algorithm) {
      case 'random':
        return await baseQuery.orderBy(sql`RANDOM()`);
      
      case 'recent':
        return await baseQuery.orderBy(desc(uploads.createdAt));
      
      case 'similarity':
        // For similarity, you'd typically need a seed embedding
        // This is simplified - you'd want to use vector similarity here
        return await baseQuery.orderBy(desc(uploads.updatedAt));
      
      default:
        return await baseQuery.orderBy(desc(uploads.createdAt));
    }
  },
};
```

## Step 9: Seed Data for Guardian

Create `src/db/seed.ts`:
```typescript
// src/db/seed.ts
import { db } from './connection';
import { users, uploads, chunks, threads } from './schema';

async function seed() {
  console.log('🌱 Seeding Guardian database...');

  try {
    // Create test user
    const [user] = await db.insert(users).values({
      email: 'guardian@example.com',
      displayName: 'Guardian User',
      preferences: { 
        theme: 'dark',
        feedAlgorithm: 'similarity',
        autoTranscribe: true,
      },
    }).returning();

    console.log('✅ Created user:', user.id);

    // Create sample uploads
    const sampleUploads = await db.insert(uploads).values([
      {
        userId: user.id,
        name: 'Morning Voice Note',
        contentType: 'audio',
        transcript: 'Today I want to focus on building the recommendation engine for Guardian. The key insight is that we need to track user interactions to improve the feed algorithm.',
        processingStatus: 'completed',
        transcriptionStatus: 'completed',
        embeddingStatus: 'completed',
      },
      {
        userId: user.id,
        name: 'Project Ideas',
        contentType: 'text',
        transcript: 'Three potential features for Guardian: 1) Swipe-to-branch threads 2) Dynamic summarization based on context 3) External content scraping from blogs and social media',
        processingStatus: 'completed',
        embeddingStatus: 'completed',
      },
      {
        userId: user.id,
        name: 'Reading Notes',
        contentType: 'text',
        transcript: 'Key takeaway from the Drizzle documentation: vector operations work seamlessly with raw SQL when needed. This is perfect for our embedding similarity searches.',
        processingStatus: 'completed',
        embeddingStatus: 'completed',
      },
    ]).returning();

    console.log(`✅ Created ${sampleUploads.length} uploads`);

    // Create sample chunks (with dummy embeddings)
    const sampleChunks = [];
    for (const upload of sampleUploads) {
      const chunk = await db.insert(chunks).values({
        uploadId: upload.id,
        content: upload.transcript!,
        chunkIndex: 0,
        totalChunks: 1,
        summary: `Summary: ${upload.name}`,
        summaryInContext: `In the context of Guardian development: ${upload.name}`,
        keywords: ['guardian', 'development', 'notes'],
        // Note: In real app, you'd generate actual embeddings here
      }).returning();
      sampleChunks.push(chunk[0]);
    }

    console.log(`✅ Created ${sampleChunks.length} chunks`);

    // Create sample thread
    const [thread] = await db.insert(threads).values({
      userId: user.id,
      name: 'Guardian Development',
      description: 'Thread focused on building Guardian features',
      threadType: 'custom_feed',
      config: {
        algorithm: 'similarity',
        weights: {
          similarity: 0.7,
          recency: 0.3,
        },
      },
    }).returning();

    console.log('✅ Created thread:', thread.id);
    console.log('🎉 Guardian seeding completed!');

  } catch (error) {
    console.error('❌ Seeding failed:', error);
    throw error;
  }
}

seed();
```

## Step 10: Test Your Setup

Create `src/db/test.ts`:
```typescript
// src/db/test.ts
import { db } from './connection';
import { users, uploads } from './schema';
import { uploadQueries } from './queries/uploads';

async function test() {
  console.log('🧪 Testing Guardian database...');

  try {
    // Test basic query
    const allUsers = await db.select().from(users);
    console.log(`📊 Found ${allUsers.length} users`);

    if (allUsers.length > 0) {
      const userId = allUsers[0].id;
      
      // Test upload queries
      const userUploads = await uploadQueries.getUserUploads(userId);
      console.log(`📁 User has ${userUploads.length} uploads`);

      // Test upload with chunks
      if (userUploads.length > 0) {
        const uploadWithChunks = await uploadQueries.getById(userUploads[0].id);
        console.log('📄 Upload with chunks:', {
          name: uploadWithChunks?.name,
          chunkCount: uploadWithChunks?.chunks?.length || 0,
        });
      }
    }

    console.log('✅ All tests passed!');
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

test();
```

Run tests:
```bash
bun run src/db/test.ts
```

## Development Workflow

```bash
# 1. Seed database
bun run db:seed

# 2. Test setup
bun run src/db/test.ts

# 3. Start development server
bun run dev

# 4. View database
bun run db:studio

# 5. Make schema changes and regenerate
bun run db:generate && bun run db:migrate
```

This setup gives you a complete, type-safe database layer optimized for your Guardian project's unique requirements: vector embeddings, thread-based feeds, and content recommendation algorithms!