import { defineCollection } from 'astro:content'
import { z } from 'astro/zod'

const docs = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    description: z.string().optional(),
    section: z.enum(['Getting started', 'Spec format', 'Reference', 'Integrations']),
    order: z.number().int().nonnegative(),
  }),
})

const examples = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    tag: z.enum(['Rust workspace', 'Polyglot', 'CI / CD', 'VS Code', 'GitHub Action', 'Languages']),
    steps: z.number().int().positive(),
    minutes: z.number().int().positive(),
    pillars: z.array(z.string()),
    description: z.string(),
    featured: z.boolean().default(false),
    draft: z.boolean().default(false),
    order: z.number().optional(),
  }),
})

const blog = defineCollection({
  type: 'content',
  schema: z.object({
    title: z.string(),
    description: z.string(),
    category: z.enum(['announce', 'plugin', 'release', 'workflow', 'tutorial']),
    date: z.date(),
    author: z.string(),
    readTime: z.number().int().positive(),
    featured: z.boolean().default(false),
    draft: z.boolean().default(false),
  }),
})

export const collections = { docs, examples, blog }
