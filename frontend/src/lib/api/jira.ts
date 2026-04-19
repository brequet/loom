import { get } from './client';

export type JiraIssue = {
  key: string;
  summary: string;
  description: string | null;
  status: string;
};

export const getJiraIssue = (key: string) =>
  get<JiraIssue>(`/jira/issues/${key}`);
