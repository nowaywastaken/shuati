export interface Problem {
  id: number;
  title: string;
  content: string;
  difficulty: Difficulty;
  category: string;
  tags: string[];
  answer?: string;
  explanation?: string;
  createdAt: string;
  updatedAt: string;
}

export enum Difficulty {
  Easy = 'Easy',
  Medium = 'Medium',
  Hard = 'Hard',
}

export interface ProblemSet {
  id: number;
  name: string;
  description: string;
  problemCount: number;
  createdAt: string;
}

export interface UserProgress {
  problemId: number;
  status: ProgressStatus;
  attempts: number;
  lastAttempt?: string;
}

export enum ProgressStatus {
  NotStarted = 'NotStarted',
  InProgress = 'InProgress',
  Completed = 'Completed',
  Reviewing = 'Reviewing',
}

export interface GreetingResponse {
  message: string;
  version: string;
  platform: string;
}
