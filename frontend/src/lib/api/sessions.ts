import type { Session } from '$shared/Session';
import type { SessionListResponse } from '$shared/SessionListResponse';
import type { CreateSessionRequest } from '$shared/CreateSessionRequest';
import { get, post } from './client';

export const listSessions = (includeTerminated = false) =>
  get<SessionListResponse>(`/sessions${includeTerminated ? '?include_terminated=true' : ''}`);
export const getSession = (id: string) => get<Session>(`/sessions/${id}`);
export const createSession = (data: CreateSessionRequest) => post<Session>('/sessions', data);
export const resumeSession = (id: string) => post<Session>(`/sessions/${id}/resume`);
export const stopSession = (id: string) => post<Session>(`/sessions/${id}/stop`);
export const terminateSession = (id: string) => post<Session>(`/sessions/${id}/terminate`);
