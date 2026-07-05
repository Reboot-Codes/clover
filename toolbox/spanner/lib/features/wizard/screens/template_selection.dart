import 'package:flutter/material.dart';

class WizardTemplateSelection extends StatelessWidget {
  const WizardTemplateSelection({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text(
            "Start with a Template",
            style: Theme.of(context).textTheme.titleLarge,
          ),
          Text("Or, pick any parts you like."),
        ],
      ),
    );
  }
}
