import { get } from './client';

export type GitLabMR = {
  iid: number;
  title: string;
  description: string | null;
  source_branch: string;
  web_url: string;
  state: string;
};

export const searchGitLabMRs = (q: string) =>
  get<GitLabMR[]>(`/gitlab/search?q=${encodeURIComponent(q)}`);
