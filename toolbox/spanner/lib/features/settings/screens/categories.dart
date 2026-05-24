import 'package:flutter/material.dart';

class SettingCategory {
  final String id;
  final String name;
  final Icon icon;
  final String description;

  const SettingCategory({
    required this.id,
    required this.name,
    required this.icon,
    required this.description,
  });
}

const List<SettingCategory> settingsCategories = [
  SettingCategory(
    id: "connection",
    name: "Connection",
    icon: Icon(Icons.wifi),
    description: "Manage connections to C.L.O.V.E.R. instances",
  ),
];

class SettingsCategories extends StatelessWidget {
  const SettingsCategories({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.all(16.0),
      child: Column(
        children: [
          TextField(
            decoration: InputDecoration(
              border: InputBorder.none,
              labelText: 'Search',
              icon: Icon(Icons.search),
            ),
          ),
          Expanded(
            child: Container(
              padding: EdgeInsets.only(top: 8.0),
              child: ListView.builder(
                itemCount: settingsCategories.length,
                itemBuilder: (context, index) {
                  final category = settingsCategories[index];
                  return ListTile(
                    leading: category.icon,
                    title: Text(category.name),
                    subtitle: Text(category.description),
                  );
                },
              ),
            ),
          ),
        ],
      ),
    );
  }
}
