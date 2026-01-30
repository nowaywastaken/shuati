import { z } from "zod";

// OpenAI Structured Outputs Schema (as per README)
export const QuestionSchema = z.object({
  question_type: z.enum(["multiple_choice", "fill_in_the_blank", "essay"]),
  stem: z.string(), // Markdown format
  options: z.array(
    z.union([
      z.string(),
      z.object({ label: z.string(), content: z.string() })
    ])
  ).optional(),
  reference_answer: z.string(), // LaTeX format
  detailed_analysis: z.array(z.string()), // Array of steps
  media_refs: z.array(z.string()).optional(), // Image paths
  knowledge_tags: z.array(z.string()).optional(), // Knowledge points
  difficulty: z.number().min(1).max(5).optional(),
});

export const QuestionBatchSchema = z.object({
  questions: z.array(QuestionSchema),
});

export type GeneratedQuestion = z.infer<typeof QuestionSchema>;
export type QuestionBatch = z.infer<typeof QuestionBatchSchema>;

// OpenAI API Configuration
interface OpenAIConfig {
  apiKey: string;
  model?: string;
  baseURL?: string;
}

let openAIConfig: OpenAIConfig | null = null;

export function configureOpenAI(config: OpenAIConfig) {
  openAIConfig = config;
  localStorage.setItem('openai_config', JSON.stringify(config));
}

export function getOpenAIConfig(): OpenAIConfig | null {
  if (openAIConfig) return openAIConfig;
  
  const stored = localStorage.getItem('openai_config');
  if (stored) {
    openAIConfig = JSON.parse(stored);
    return openAIConfig;
  }
  
  return null;
}

// JSON Schema for OpenAI Structured Outputs
const questionJsonSchema = {
  type: "object",
  properties: {
    questions: {
      type: "array",
      items: {
        type: "object",
        properties: {
          question_type: {
            type: "string",
            enum: ["multiple_choice", "fill_in_the_blank", "essay"],
            description: "Type of the question"
          },
          stem: {
            type: "string",
            description: "Question stem in Markdown format, may contain LaTeX formulas"
          },
          options: {
            type: "array",
            items: {
              anyOf: [
                { type: "string" },
                {
                  type: "object",
                  properties: {
                    label: { type: "string" },
                    content: { type: "string" }
                  },
                  required: ["label", "content"]
                }
              ]
            },
            description: "Options for multiple choice questions"
          },
          reference_answer: {
            type: "string",
            description: "Reference answer in LaTeX format"
          },
          detailed_analysis: {
            type: "array",
            items: { type: "string" },
            description: "Detailed analysis steps, each step is a string"
          },
          media_refs: {
            type: "array",
            items: { type: "string" },
            description: "References to media files (images, etc.)"
          },
          knowledge_tags: {
            type: "array",
            items: { type: "string" },
            description: "Knowledge tags for categorization"
          },
          difficulty: {
            type: "integer",
            minimum: 1,
            maximum: 5,
            description: "Difficulty level from 1 to 5"
          }
        },
        required: ["question_type", "stem", "reference_answer", "detailed_analysis"]
      }
    }
  },
  required: ["questions"]
};

// Multi-agent pipeline for quality assurance
export async function generateQuestionsFromText(text: string): Promise<GeneratedQuestion[]> {
  const config = getOpenAIConfig();
  
  // If no API key configured, use mock data
  if (!config?.apiKey) {
    console.log("No OpenAI config found, using mock data");
    return generateMockQuestions(text);
  }
  
  try {
    // Step 1: Generator Agent - Generate initial questions
    const generated = await callOpenAIGenerator(text, config);
    
    // Step 2: Verifier Agent - Verify quality
    const verified = await verifyQuestions(generated, config);
    
    // Step 3: Formatter Agent - Final formatting
    return formatQuestions(verified);
    
  } catch (error) {
    console.error("AI generation failed, falling back to mock:", error);
    return generateMockQuestions(text);
  }
}

async function callOpenAIGenerator(text: string, config: OpenAIConfig): Promise<GeneratedQuestion[]> {
  const response = await fetch(`${config.baseURL || 'https://api.openai.com/v1'}/chat/completions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${config.apiKey}`
    },
    body: JSON.stringify({
      model: config.model || 'gpt-4o',
      messages: [
        {
          role: 'system',
          content: `You are an expert educational content generator. Analyze the provided text and generate high-quality practice questions.
          
Guidelines:
1. Generate diverse question types (multiple choice, fill-in-blank, essay)
2. Include LaTeX formulas for mathematical content using $...$ or $$...$$
3. Provide detailed step-by-step analysis
4. Tag questions with relevant knowledge points
5. Rate difficulty from 1 (easy) to 5 (hard)

Return questions in the specified JSON format.`
        },
        {
          role: 'user',
          content: `Generate questions from the following text:\n\n${text}`
        }
      ],
      response_format: {
        type: "json_schema",
        json_schema: {
          name: "question_batch",
          schema: questionJsonSchema,
          strict: true
        }
      }
    })
  });
  
  if (!response.ok) {
    throw new Error(`OpenAI API error: ${response.statusText}`);
  }
  
  const data = await response.json();
  const parsed = QuestionBatchSchema.parse(JSON.parse(data.choices[0].message.content));
  return parsed.questions;
}

async function verifyQuestions(questions: GeneratedQuestion[], _config: OpenAIConfig): Promise<GeneratedQuestion[]> {
  // For now, simple verification - in production would call another agent
  return questions.filter(q => {
    // Basic validation
    if (!q.stem || q.stem.length < 10) return false;
    if (!q.reference_answer) return false;
    if (q.question_type === 'multiple_choice' && (!q.options || q.options.length < 2)) return false;
    return true;
  });
}

function formatQuestions(questions: GeneratedQuestion[]): GeneratedQuestion[] {
  // Ensure consistent formatting
  return questions.map(q => ({
    ...q,
    stem: q.stem.trim(),
    reference_answer: q.reference_answer.trim(),
    detailed_analysis: q.detailed_analysis.map(step => step.trim()),
  }));
}

// Mock implementation for development
function generateMockQuestions(text: string): GeneratedQuestion[] {
  console.log("Generating mock questions from text length:", text.length);
  
  // Generate different questions based on content type
  if (text.toLowerCase().includes('algorithm') || text.toLowerCase().includes('sort')) {
    return [
      {
        question_type: "multiple_choice",
        stem: "What is the **time complexity** of QuickSort in the worst case?",
        options: [
          { label: "A", content: "O(n \\log n)" },
          { label: "B", content: "O(n^2)" },
          { label: "C", content: "O(n)" },
          { label: "D", content: "O(\\log n)" }
        ],
        reference_answer: "B",
        detailed_analysis: [
          "QuickSort has a worst-case time complexity of $O(n^2)$.",
          "This happens when the pivot selection is poor (e.g., always picking the smallest or largest element in a sorted array).",
          "However, its average case is $O(n \\log n)$."
        ],
        knowledge_tags: ["algorithms", "sorting", "time-complexity"],
        difficulty: 3
      },
      {
        question_type: "fill_in_the_blank",
        stem: "The space complexity of MergeSort is $O($______$)$.",
        reference_answer: "n",
        detailed_analysis: [
          "MergeSort requires auxiliary space proportional to the input size.",
          "This is because it needs to merge two sorted subarrays into a temporary array."
        ],
        knowledge_tags: ["algorithms", "sorting", "space-complexity"],
        difficulty: 2
      }
    ];
  }
  
  if (text.toLowerCase().includes('physics') || text.toLowerCase().includes('energy')) {
    return [
      {
        question_type: "fill_in_the_blank",
        stem: "The formula for mass-energy equivalence is $E = mc^2$. Here $c$ stands for ______.",
        reference_answer: "\\text{speed of light}",
        detailed_analysis: [
          "In physics, mass–energy equivalence is the relationship between mass and energy in a system's rest frame.",
          "$c$ represents the speed of light in vacuum, approximately $3 \\times 10^8$ m/s."
        ],
        knowledge_tags: ["physics", "relativity", "mass-energy"],
        difficulty: 2
      },
      {
        question_type: "essay",
        stem: "Explain the significance of Einstein's equation $E = mc^2$ in modern physics.",
        reference_answer: "The equation establishes that mass and energy are interchangeable. It explains nuclear reactions, forms the basis of nuclear energy, and is fundamental to understanding stellar energy production.",
        detailed_analysis: [
          "Step 1: Explain the mathematical meaning - mass can be converted to energy and vice versa.",
          "Step 2: Discuss nuclear applications - fission and fusion reactions.",
          "Step 3: Mention astronomical implications - stellar nucleosynthesis.",
          "Step 4: Note the small conversion factor $c^2$ means small mass yields huge energy."
        ],
        knowledge_tags: ["physics", "relativity", "nuclear-physics"],
        difficulty: 4
      }
    ];
  }
  
  // Default questions
  return [
    {
      question_type: "multiple_choice",
      stem: "Which of the following best describes the content provided?",
      options: [
        { label: "A", content: "Technical documentation" },
        { label: "B", content: "Educational material" },
        { label: "C", content: "Research paper" },
        { label: "D", content: "Need more context to determine" }
      ],
      reference_answer: "D",
      detailed_analysis: [
        "The content type cannot be determined without specific subject matter indicators.",
        "Please provide content with clear educational or technical markers."
      ],
      knowledge_tags: ["general"],
      difficulty: 1
    }
  ];
}

// Batch API support for large document processing (50% cost savings)
export async function submitBatchJob(_documentText: string): Promise<string | null> {
  const config = getOpenAIConfig();
  if (!config?.apiKey) return null;
  
  // Note: In production, documentText would be uploaded to OpenAI files API first
  // then referenced by file_id in the batch request
  
  try {
    const response = await fetch(`${config.baseURL || 'https://api.openai.com/v1'}/batches`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${config.apiKey}`
      },
      body: JSON.stringify({
        input_file_id: "file_placeholder", // Would need to upload file first
        endpoint: "/v1/chat/completions",
        completion_window: "24h"
      })
    });
    
    if (!response.ok) {
      console.error('Batch API not available:', response.statusText);
      return null;
    }
    
    const data = await response.json();
    return data.id;
  } catch (error) {
    console.error('Batch submission failed:', error);
    return null;
  }
}
