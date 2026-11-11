import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';
import { tap, map } from 'rxjs/operators';

export interface Movie {
  id?: number;
  title: string;
  description?: string;
  posterUrl?: string;
  year?: number;
  genre?: string;
  director?: string;
  runtime?: number;
  vote_average?: number;
  budget?: number;
}

@Injectable({
    providedIn: 'root'
})
export class ApiService {
    private readonly API_BASE = 'http://localhost:3000/api/v1';

    constructor(private http: HttpClient) {}

    login(credentials: any): Observable<any> {
        return this.http.post(`${this.API_BASE}/users/login`, credentials).pipe(
            tap((res: any) => {
                if (typeof window !== 'undefined' && window.localStorage && res && res.token) {
                    localStorage.setItem('token', res.token);
                }
            })
        );
    }

    signup(user: any): Observable<any> {
        return this.http.post(`${this.API_BASE}/users/bootstrap`, user);
    }

    getMovies(limit: number = 20, offset: number = 0): Observable<Movie[]> {
        const token = (typeof window !== 'undefined' && window.localStorage) ? localStorage.getItem('token') : null;
        let headers = new HttpHeaders();
        if (token) {
            headers = headers.set('Authorization', `Bearer ${token}`);
        }
        const url = `${this.API_BASE}/movies?limit=${limit}&offset=${offset}`;
        return this.http.get<any>(url, { headers }).pipe(
            map((response: any) => {
                // El backend devuelve { data: [...], Count: N }
                const movies: any[] = Array.isArray(response) ? response : (response.data || []);
                return movies.map(m => ({
                    id: m.id,
                    title: m.title,
                    description: m.overview || m.description || '',
                    year: m.release_date
                        ? parseInt(String(m.release_date).split('-')[0], 10)
                        : m.year,
                    genre: m.genres && m.genres.length > 0
                        ? (typeof m.genres[0] === 'string' ? m.genres[0] : m.genres[0].name)
                        : (m.genre || 'Desconocido'),
                    posterUrl: m.poster_url || m.posterUrl || null,
                    director: m.director,
                    runtime: m.runtime,
                    vote_average: m.vote_average,
                    budget: m.budget
                }));
            })
        );
    }

    getMovieById(id: number): Observable<Movie> {
        const token = (typeof window !== 'undefined' && window.localStorage) ? localStorage.getItem('token') : null;
        let headers = new HttpHeaders();
        if (token) {
            headers = headers.set('Authorization', `Bearer ${token}`);
        }
        return this.http.get<any>(`${this.API_BASE}/movies/${id}`, { headers }).pipe(
            map((m: any) => ({
                id: m.id,
                title: m.title,
                description: m.overview || m.description || '',
                year: m.release_date
                    ? parseInt(String(m.release_date).split('-')[0], 10)
                    : m.year,
                genre: m.genres && m.genres.length > 0
                    ? (typeof m.genres[0] === 'string' ? m.genres[0] : m.genres[0].name)
                    : (m.genre || 'Desconocido'),
                posterUrl: m.poster_url || m.posterUrl || null,
                director: m.director,
                runtime: m.runtime,
                vote_average: m.vote_average,
                budget: m.budget
            }))
        );
    }

    logout() {
        if (typeof window !== 'undefined' && window.localStorage) {
            localStorage.removeItem('token');
        }
    }

    isLoggedIn(): boolean {
        if (typeof window !== 'undefined' && window.localStorage) {
            return !!localStorage.getItem('token');
        }
        return false;
    }
}