import 'package:flutter/material.dart';

class WizardPartFlashing extends StatelessWidget {
  const WizardPartFlashing({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text("Flashing: ...", style: Theme.of(context).textTheme.titleLarge),
        ],
      ),
    );
  }
}
