import type { Session } from '$shared/Session';
import { get, post } from './client';

export const listSessions = () => get<{ sessions: Session[] }>('/sessions');
export const getSession = (id: string) => get<Session>(`/sessions/${id}`);
export const createSession = (data: {
  source_type: string;
  source_ref?: string;
  project_id?: string;
  title?: string;
  model?: string;
  custom_instructions?: string;
}) => post<Session>('/sessions', data);
export const resumeSession = (id: string) => post<Session>(`/sessions/${id}/resume`);
export const stopSession = (id: string) => post<Session>(`/sessions/${id}/stop`);
export const terminateSession = (id: string) => post<Session>(`/sessions/${id}/terminate`);
