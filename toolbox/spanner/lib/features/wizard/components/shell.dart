import 'package:flutter/material.dart';
import 'package:spanner/components/window_controls.dart';

class WizardShell extends StatelessWidget {
  final Widget child;

  const WizardShell({super.key, required this.child});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      // TODO: Add progress bar.
      appBar: AppBar(title: Text("Setup Wizard"), actions: [WindowControls()]),
      body: child,
    );
  }
}
