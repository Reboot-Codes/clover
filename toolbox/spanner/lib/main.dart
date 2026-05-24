import 'package:flutter/material.dart';
import 'router.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp.router(
      routerConfig: router,
      title: 'Spanner',
      theme: ThemeData(
        brightness: Brightness.light,
        colorSchemeSeed: const Color(0xFF7CCF9E),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        brightness: Brightness.dark,
        colorSchemeSeed: const Color(0xFF7CCF9E),
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
    );
  }
}
