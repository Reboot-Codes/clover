import 'package:flutter/material.dart';

class WizardPartConfirmation extends StatelessWidget {
  const WizardPartConfirmation({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text(
            "Module Build Steps",
            style: Theme.of(context).textTheme.titleLarge,
          ),
          Text(
            "Ensure that all the modules listed here meet your needs because the fun part is here! We've prepared a printable bill of materials, sourcing tips, and build instructions just for your setup.",
          ),
        ],
      ),
    );
  }
}
