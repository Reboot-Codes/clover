import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:spanner/components/window_controls.dart';

class SettingsShell extends StatelessWidget {
  final Widget child;

  const SettingsShell({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.arrow_back),
          onPressed: () {
            context.pop();
          },
        ),
        title: Text("Settings"),
        actions: [WindowControls()],
      ),
      body: child,
    );
  }
}
