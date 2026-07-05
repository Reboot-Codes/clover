import 'package:flutter/material.dart';

class WizardInstanceConnection extends StatelessWidget {
  const WizardInstanceConnection({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text(
            "Connecting to your instance...",
            style: Theme.of(context).textTheme.titleLarge,
          ),
        ],
      ),
    );
  }
}
