import { Component } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Router } from '@angular/router';
import { ApiService } from '../api.service';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [FormsModule, CommonModule],
  templateUrl: './login.html',
  styleUrl: './login.css',
})
export class Login {
  username = '';
  password = '';
  errorMessage = '';
  isLoading = false;

  constructor(private apiService: ApiService, private router: Router) {}

  login() {
    this.errorMessage = '';
    if (!this.username || !this.password) {
      this.errorMessage = 'Por favor, ingrese usuario y contraseña.';
      return;
    }
    this.isLoading = true;
    this.apiService.login({ username: this.username, password: this.password }).subscribe({
      next: () => {
        this.isLoading = false;
        this.router.navigate(['/']);
      },
      error: (err) => {
        this.isLoading = false;
        this.errorMessage = 'Usuario o contraseña incorrectos.';
        console.error(err);
      }
    });
  }
}
