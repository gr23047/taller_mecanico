import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:shared_preferences/shared_preferences.dart';

class ApiConfig {
  /// - Celular físico: IP de tu PC en la wifi (hostname -I)
  /// - Emulador Android: http://10.0.2.2:8090
  /// - Linux/Windows desktop o Chrome: http://localhost:8090
  static const String baseUrl = 'http://192.168.1.22:8090';
  static const Duration timeout = Duration(seconds: 10);
}

class ApiException implements Exception {
  final String message;
  final int? status;
  ApiException(this.message, [this.status]);

  @override
  String toString() => message;
}

class Session {
  final String token;
  final String nombre;
  final String email;
  final String rol;

  const Session({
    required this.token,
    required this.nombre,
    required this.email,
    required this.rol,
  });
}

class ApiService {
  static const _kToken = 'token';

  Future<Map<String, dynamic>> _request(
    String method,
    String path, {
    Map<String, dynamic>? body,
    String? token,
  }) async {
    final uri = Uri.parse('${ApiConfig.baseUrl}$path');
    final headers = {
      'Content-Type': 'application/json',
      if (token != null) 'Authorization': 'Bearer $token',
    };

    http.Response res;
    try {
      res = method == 'GET'
          ? await http.get(uri, headers: headers).timeout(ApiConfig.timeout)
          : await http
              .post(uri, headers: headers, body: jsonEncode(body))
              .timeout(ApiConfig.timeout);
    } on TimeoutException {
      throw ApiException(
          'El servidor no responde. Revisa la IP y que la API esté corriendo.');
    } catch (_) {
      throw ApiException('No se pudo conectar con el servidor.');
    }

    Map<String, dynamic> data = {};
    try {
      data = jsonDecode(utf8.decode(res.bodyBytes)) as Map<String, dynamic>;
    } catch (_) {}

    if (res.statusCode >= 200 && res.statusCode < 300) return data;
    throw ApiException(
      data['error'] as String? ?? 'Error del servidor (${res.statusCode})',
      res.statusCode,
    );
  }

  Future<void> register({
    required String nombre,
    required String email,
    required String password,
  }) async {
    await _request('POST', '/auth/register', body: {
      'nombre': nombre.trim(),
      'email': email.trim(),
      'password': password,
    });
  }

  Future<Session> login({
    required String email,
    required String password,
  }) async {
    final data = await _request('POST', '/auth/login', body: {
      'email': email.trim(),
      'password': password,
    });
    final u = data['usuario'] as Map<String, dynamic>;
    final session = Session(
      token: data['token'] as String,
      nombre: u['nombre'] as String,
      email: u['email'] as String,
      rol: u['rol'] as String,
    );
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kToken, session.token);
    return session;
  }

  /// Si hay un token guardado y sigue siendo válido, devuelve la sesión.
  Future<Session?> restoreSession() async {
    final prefs = await SharedPreferences.getInstance();
    final token = prefs.getString(_kToken);
    if (token == null) return null;
    try {
      final d = await _request('GET', '/auth/me', token: token);
      return Session(
        token: token,
        nombre: d['nombre'] as String,
        email: d['email'] as String,
        rol: d['rol'] as String,
      );
    } on ApiException catch (e) {
      if (e.status == 401) await prefs.remove(_kToken);
      return null;
    }
  }

  Future<void> logout() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(_kToken);
  }
}
