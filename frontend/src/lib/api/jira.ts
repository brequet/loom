import { get } from './client';

export type JiraIssue = {
  key: string;
  summary: string;
  description: string | null;
  status: string;
};

export const searchJiraIssues = (q: string) =>
  get<JiraIssue[]>(`/jira/search?q=${encodeURIComponent(q)}`);

export const getJiraIssue = (key: string) =>
  get<JiraIssue>(`/jira/issues/${key}`);
