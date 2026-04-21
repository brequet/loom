import { get } from './client';
import type { GitLabMergeRequest } from '$shared/GitLabMergeRequest';

export type { GitLabMergeRequest };

export const searchGitLabMRs = (q: string) =>
  get<GitLabMergeRequest[]>(`/gitlab/search?q=${encodeURIComponent(q)}`);
