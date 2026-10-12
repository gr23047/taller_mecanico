import 'package:flutter/material.dart';

import 'api_service.dart';

void main() => runApp(const TallerApp());

class TallerApp extends StatelessWidget {
  const TallerApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Taller Mecánico',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(colorSchemeSeed: Colors.indigo, useMaterial3: true),
      home: const Root(),
    );
  }
}

/// Decide si mostrar el login o el inicio según haya una sesión válida guardada.
class Root extends StatefulWidget {
  const Root({super.key});

  @override
  State<Root> createState() => _RootState();
}

class _RootState extends State<Root> {
  late final Future<Session?> _session = ApiService().restoreSession();

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<Session?>(
      future: _session,
      builder: (context, snap) {
        if (snap.connectionState != ConnectionState.done) {
          return const Scaffold(body: Center(child: CircularProgressIndicator()));
        }
        final s = snap.data;
        return s == null ? const AuthPage() : HomePage(session: s);
      },
    );
  }
}

class AuthPage extends StatefulWidget {
  const AuthPage({super.key});

  @override
  State<AuthPage> createState() => _AuthPageState();
}

class _AuthPageState extends State<AuthPage> {
  final _formKey = GlobalKey<FormState>();
  final _nameCtrl = TextEditingController();
  final _emailCtrl = TextEditingController();
  final _passCtrl = TextEditingController();
  final _api = ApiService();

  bool _isLogin = true;
  bool _loading = false;
  bool _hidePass = true;

  @override
  void dispose() {
    _nameCtrl.dispose();
    _emailCtrl.dispose();
    _passCtrl.dispose();
    super.dispose();
  }

  void _snack(String msg, {bool error = false}) {
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(
      content: Text(msg),
      backgroundColor: error ? Colors.red.shade700 : null,
    ));
  }

  Future<void> _submit() async {
    if (!_formKey.currentState!.validate()) return;
    setState(() => _loading = true);
    try {
      if (_isLogin) {
        final session = await _api.login(
          email: _emailCtrl.text,
          password: _passCtrl.text,
        );
        if (!mounted) return;
        Navigator.of(context).pushReplacement(
          MaterialPageRoute(builder: (_) => HomePage(session: session)),
        );
      } else {
        await _api.register(
          nombre: _nameCtrl.text,
          email: _emailCtrl.text,
          password: _passCtrl.text,
        );
        if (!mounted) return;
        _snack('Cuenta creada. Ya puedes iniciar sesión.');
        setState(() => _isLogin = true);
        _passCtrl.clear();
      }
    } on ApiException catch (e) {
      if (mounted) _snack(e.message, error: true);
    } finally {
      if (mounted) setState(() => _loading = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme.primary;
    return Scaffold(
      body: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 400),
            child: Form(
              key: _formKey,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(Icons.build_circle_outlined, size: 72, color: color),
                  const SizedBox(height: 12),
                  Text('Taller Mecánico',
                      style: Theme.of(context).textTheme.headlineMedium),
                  const SizedBox(height: 4),
                  Text(_isLogin ? 'Inicia sesión' : 'Crea tu cuenta'),
                  const SizedBox(height: 24),
                  if (!_isLogin) ...[
                    TextFormField(
                      controller: _nameCtrl,
                      textCapitalization: TextCapitalization.words,
                      decoration: const InputDecoration(
                        labelText: 'Nombre',
                        prefixIcon: Icon(Icons.person_outline),
                        border: OutlineInputBorder(),
                      ),
                      validator: (v) => (v == null || v.trim().isEmpty)
                          ? 'Ingresa tu nombre'
                          : null,
                    ),
                    const SizedBox(height: 16),
                  ],
                  TextFormField(
                    controller: _emailCtrl,
                    keyboardType: TextInputType.emailAddress,
                    decoration: const InputDecoration(
                      labelText: 'Correo',
                      prefixIcon: Icon(Icons.email_outlined),
                      border: OutlineInputBorder(),
                    ),
                    validator: (v) {
                      final t = v?.trim() ?? '';
                      if (t.isEmpty) return 'Ingresa tu correo';
                      if (!RegExp(r'^[^@\s]+@[^@\s]+\.[^@\s]+$').hasMatch(t)) {
                        return 'Correo no válido';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 16),
                  TextFormField(
                    controller: _passCtrl,
                    obscureText: _hidePass,
                    decoration: InputDecoration(
                      labelText: 'Contraseña',
                      prefixIcon: const Icon(Icons.lock_outline),
                      border: const OutlineInputBorder(),
                      suffixIcon: IconButton(
                        icon: Icon(_hidePass
                            ? Icons.visibility_outlined
                            : Icons.visibility_off_outlined),
                        onPressed: () => setState(() => _hidePass = !_hidePass),
                      ),
                    ),
                    validator: (v) {
                      if (v == null || v.isEmpty) return 'Ingresa tu contraseña';
                      if (!_isLogin && v.length < 8) {
                        return 'Mínimo 8 caracteres';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 24),
                  SizedBox(
                    width: double.infinity,
                    height: 48,
                    child: FilledButton(
                      onPressed: _loading ? null : _submit,
                      child: _loading
                          ? const SizedBox(
                              width: 22,
                              height: 22,
                              child: CircularProgressIndicator(strokeWidth: 2),
                            )
                          : Text(_isLogin ? 'Entrar' : 'Registrarme'),
                    ),
                  ),
                  const SizedBox(height: 8),
                  TextButton(
                    onPressed: _loading
                        ? null
                        : () => setState(() => _isLogin = !_isLogin),
                    child: Text(_isLogin
                        ? '¿No tienes cuenta? Regístrate'
                        : '¿Ya tienes cuenta? Inicia sesión'),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class HomePage extends StatelessWidget {
  final Session session;
  const HomePage({super.key, required this.session});

  Future<void> _logout(BuildContext context) async {
    await ApiService().logout();
    if (!context.mounted) return;
    Navigator.of(context).pushAndRemoveUntil(
      MaterialPageRoute(builder: (_) => const AuthPage()),
      (_) => false,
    );
  }

  @override
  Widget build(BuildContext context) {
    const modulos = [
      (Icons.directions_car_outlined, 'Vehículos y clientes'),
      (Icons.notifications_active_outlined, 'Alertas de mantenimiento'),
      (Icons.assignment_outlined, 'Órdenes de trabajo'),
      (Icons.inventory_2_outlined, 'Repuestos'),
    ];

    return Scaffold(
      appBar: AppBar(
        title: const Text('Taller Mecánico'),
        actions: [
          IconButton(
            icon: const Icon(Icons.logout),
            tooltip: 'Cerrar sesión',
            onPressed: () => _logout(context),
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text('Hola, ${session.nombre}',
              style: Theme.of(context).textTheme.headlineSmall),
          Text('${session.email} · ${session.rol}'),
          const SizedBox(height: 16),
          for (final m in modulos)
            Card(
              child: ListTile(
                leading: Icon(m.$1),
                title: Text(m.$2),
                subtitle: const Text('Próximamente'),
                enabled: false,
              ),
            ),
        ],
      ),
    );
  }
}
