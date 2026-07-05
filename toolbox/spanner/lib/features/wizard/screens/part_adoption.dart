import 'package:flutter/material.dart';

class WizardPartAdoption extends StatelessWidget {
  const WizardPartAdoption({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: .only(left: 16, right: 16, top: 16),
      child: Column(
        crossAxisAlignment: .start,
        children: [
          Text(
            "Adopt your new Modules",
            style: Theme.of(context).textTheme.titleLarge,
          ),
          Text(
            "Each module may have a different paring process, we've prepared a checklist so you don't miss any of them.",
          ),
        ],
      ),
    );
  }
}
