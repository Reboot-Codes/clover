import 'package:flutter/material.dart';

class WizardPartAssembly extends StatelessWidget {
  const WizardPartAssembly({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text("Assemble: ...", style: Theme.of(context).textTheme.titleLarge),
        ],
      ),
    );
  }
}
