import 'package:flutter/material.dart';
import 'router.dart';

void main() {
  // TODO: Custom window decorations when running on desktop.
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // TODO: Match B.E.N.T.O. more closely.
  ThemeData _generateTheme(bool isDark) {
    return ThemeData(
      brightness: isDark ? Brightness.dark : Brightness.light,
      colorSchemeSeed: const Color(0xFF7CCF9E),
      useMaterial3: true,
    );
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      routerConfig: router,
      title: 'Spanner',
      theme: _generateTheme(false),
      darkTheme: _generateTheme(true),
      themeMode: ThemeMode.system,
    );
  }
}
