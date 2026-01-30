import { invoke } from '@tauri-apps/api/core';

export interface Question {
  id?: number;
  question_type: 'multiple_choice' | 'fill_in_the_blank' | 'essay';
  stem: string;
  options?: string | null; // JSON string
  reference_answer: string;
  detailed_analysis: string;
  media_refs?: string | null; // JSON string
  knowledge_tags?: string | null; // JSON string
  difficulty?: number | null;
  created_at?: string;
}

export interface QuestionAttempt {
  id?: number;
  question_id: number;
  user_answer: string;
  is_correct: boolean;
  confidence_score?: number;
  time_spent_seconds: number;
  created_at?: string;
}

export interface BatchImportResult {
  success: boolean;
  imported_count: number;
  errors: string[];
}

export async function initDb(): Promise<void> {
  await invoke('init_database');
}

export async function batchImportQuestions(questions: Question[]): Promise<BatchImportResult> {
  return await invoke('batch_import_questions', { questions });
}

export async function getAllQuestions(): Promise<Question[]> {
  return await invoke('get_all_questions');
}

export async function saveAttempt(attempt: QuestionAttempt): Promise<number> {
  return await invoke('save_attempt', { attempt });
}

export async function getMistakesByTag(tag?: string): Promise<Array<[Question, number]>> {
  return await invoke('get_mistakes_by_tag', { tag });
}
