import { get } from './client';
import type { JiraIssue } from '$shared/JiraIssue';

export const getJiraIssue = (key: string) =>
  get<JiraIssue>(`/jira/issues/${key}`);
