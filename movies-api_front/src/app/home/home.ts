import { Component, OnInit, HostListener } from '@angular/core';
import { ApiService, Movie } from '../api.service';
import { CommonModule } from '@angular/common';
import { Router } from '@angular/router';

@Component({
  selector: 'app-home',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './home.html',
  styleUrl: './home.css',
})
export class Home implements OnInit {
  movies: Movie[] = [];
  isLoading = true;
  isLoadingMore = false;
  error = '';
  isLoggedIn = false;
  currentOffset = 0;
  readonly PAGE_SIZE = 20;
  hasMore = true;

  // Modal state
  selectedMovie: Movie | null = null;
  isModalLoading = false;
  isModalVisible = false;

  constructor(private apiService: ApiService, private router: Router) {}

  ngOnInit(): void {
    this.isLoggedIn = this.apiService.isLoggedIn();
    this.fetchMovies();
  }

  goToLogin() {
    this.router.navigate(['/login']);
  }

  fetchMovies() {
    this.isLoading = true;
    this.error = '';
    this.apiService.getMovies(this.PAGE_SIZE, 0).subscribe({
      next: (data) => {
        this.movies = data;
        this.currentOffset = data.length;
        this.hasMore = data.length === this.PAGE_SIZE;
        this.isLoading = false;
      },
      error: (err) => {
        console.error('Error al obtener peliculas:', err);
        this.error = `No se pudo conectar con el servidor (${err.status || 'sin conexion'}). Verifica que el backend este corriendo en http://localhost:3000`;
        this.isLoading = false;
      }
    });
  }

  loadMore() {
    if (this.isLoadingMore || !this.hasMore) return;
    this.isLoadingMore = true;
    this.apiService.getMovies(this.PAGE_SIZE, this.currentOffset).subscribe({
      next: (data) => {
        this.movies = [...this.movies, ...data];
        this.currentOffset += data.length;
        this.hasMore = data.length === this.PAGE_SIZE;
        this.isLoadingMore = false;
      },
      error: (err) => {
        console.error('Error al cargar mas peliculas:', err);
        this.isLoadingMore = false;
      }
    });
  }

  openMovieDetail(movie: Movie) {
    if (!movie.id) {
      this.selectedMovie = movie;
      this.isModalVisible = true;
      document.body.style.overflow = 'hidden';
      return;
    }
    this.isModalLoading = true;
    this.isModalVisible = true;
    this.selectedMovie = movie; // show instantly, then enrich
    document.body.style.overflow = 'hidden';

    this.apiService.getMovieById(movie.id).subscribe({
      next: (detail) => {
        this.selectedMovie = detail;
        this.isModalLoading = false;
      },
      error: () => {
        // fallback to card data already set
        this.isModalLoading = false;
      }
    });
  }

  closeModal() {
    this.isModalVisible = false;
    document.body.style.overflow = '';
    setTimeout(() => { this.selectedMovie = null; }, 300);
  }

  @HostListener('document:keydown.escape')
  onEscapeKey() {
    if (this.isModalVisible) this.closeModal();
  }

  getPosterFallback(movie: Movie): string {
    const genreImages: { [key: string]: string } = {
      'Action': 'https://images.unsplash.com/photo-1509347528160-9a9e33742cdb?q=80&w=800&auto=format&fit=crop',
      'Adventure': 'https://images.unsplash.com/photo-1520716197184-9bce4e776bc1?q=80&w=800&auto=format&fit=crop',
      'Animation': 'https://images.unsplash.com/photo-1534447677768-be436bb09401?q=80&w=800&auto=format&fit=crop',
      'Comedy': 'https://images.unsplash.com/photo-1485095329183-d0797cdc5676?q=80&w=800&auto=format&fit=crop',
      'Crime': 'https://images.unsplash.com/photo-1545552987-720aa18145ca?q=80&w=800&auto=format&fit=crop',
      'Documentary': 'https://images.unsplash.com/photo-1485846234645-a62644f84728?q=80&w=800&auto=format&fit=crop',
      'Drama': 'https://images.unsplash.com/photo-1440404653325-ab127d49abc1?q=80&w=800&auto=format&fit=crop',
      'Fantasy': 'https://images.unsplash.com/photo-1518709268805-4e9042af9f23?q=80&w=800&auto=format&fit=crop',
      'Horror': 'https://images.unsplash.com/photo-1509248961158-e54f6934749c?q=80&w=800&auto=format&fit=crop',
      'Mystery': 'https://images.unsplash.com/photo-1518709268805-4e9042af9f23?q=80&w=800&auto=format&fit=crop',
      'Romance': 'https://images.unsplash.com/photo-1518199266791-5375a83190b7?q=80&w=800&auto=format&fit=crop',
      'Science Fiction': 'https://images.unsplash.com/photo-1446776811953-b23d57bd21aa?q=80&w=800&auto=format&fit=crop',
      'Thriller': 'https://images.unsplash.com/photo-1533613220915-609f661a6fe1?q=80&w=800&auto=format&fit=crop',
      'War': 'https://images.unsplash.com/photo-1549482159-cf7f1a53f15e?q=80&w=800&auto=format&fit=crop',
      'Western': 'https://images.unsplash.com/photo-1516557070061-c3d1653fa646?q=80&w=800&auto=format&fit=crop',
    };
    const defaultImg = 'https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?q=80&w=800&auto=format&fit=crop';
    return genreImages[movie.genre || ''] || defaultImg;
  }

  formatBudget(budget?: number): string {
    if (!budget || budget === 0) return 'No disponible';
    if (budget >= 1_000_000) return `$${(budget / 1_000_000).toFixed(1)}M`;
    if (budget >= 1_000) return `$${(budget / 1_000).toFixed(0)}K`;
    return `$${budget}`;
  }

  logout() {
    this.apiService.logout();
    this.isLoggedIn = false;
  }
}