import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:spanner/components/window_controls.dart';

class NavDestinationConfig {
  final String id; // Unique identifier for reordering/keys
  final String label; // Text shown below the icon
  final String routePath; // The router path (base) target
  final IconData icon; // Default outlined icon
  final IconData selectedIcon; // Filled icon when active

  const NavDestinationConfig({
    required this.id,
    required this.label,
    required this.routePath,
    required this.icon,
    required this.selectedIcon,
  });
}

// Your default layout ordering blueprint
const List<NavDestinationConfig> defaultNavDestinations = [
  NavDestinationConfig(
    id: 'overview',
    label: 'Overview',
    routePath: '/repo_manager',
    icon: Icons.settings_outlined,
    selectedIcon: Icons.settings,
  ),
  NavDestinationConfig(
    id: 'modules',
    label: 'Modules',
    routePath: '/repo_manager/modules',
    icon: Icons.extension_outlined,
    selectedIcon: Icons.extension,
  ),
  NavDestinationConfig(
    id: 'gestures',
    label: 'Gestures',
    routePath: '/repo_manager/gestures',
    icon: Icons.gesture_outlined,
    selectedIcon: Icons.gesture,
  ),
  NavDestinationConfig(
    id: 'apps',
    label: 'Apps',
    routePath: '/repo_manager/apps',
    icon: Icons.code_outlined,
    selectedIcon: Icons.code,
  ),
];

class RepoManagerShell extends StatelessWidget {
  final StatefulNavigationShell navigationShell;
  // TODO: Store the order of navbar destinations in settings
  final List<NavDestinationConfig> activeDestinations = defaultNavDestinations;

  const RepoManagerShell({super.key, required this.navigationShell});

  void _onTabSelected(int index) {
    navigationShell.goBranch(
      index,
      initialLocation: index == navigationShell.currentIndex,
    );
  }

  @override
  Widget build(BuildContext context) {
    final int selectedIndex = navigationShell.currentIndex;

    return LayoutBuilder(
      builder: (context, constraints) {
        final bool isSmall = constraints.maxWidth < 600;

        return Scaffold(
          appBar: AppBar(
            leading: BackButton(onPressed: () => context.go("/")),
            title: Text("Spanner | Repo"),
            actions: [WindowControls()],
          ),
          bottomNavigationBar: isSmall
              ? NavigationBar(
                  selectedIndex: selectedIndex,
                  onDestinationSelected: _onTabSelected,
                  destinations: activeDestinations.map((dest) {
                    return NavigationDestination(
                      icon: Icon(dest.icon),
                      selectedIcon: Icon(dest.selectedIcon),
                      label: dest.label,
                    );
                  }).toList(),
                )
              : null,

          body: Row(
            children: [
              if (!isSmall) ...[
                NavigationRail(
                  selectedIndex: selectedIndex,
                  onDestinationSelected: _onTabSelected,
                  labelType: NavigationRailLabelType.all,
                  destinations: activeDestinations.map((dest) {
                    return NavigationRailDestination(
                      icon: Icon(dest.icon),
                      selectedIcon: Icon(dest.selectedIcon),
                      label: Text(dest.label),
                    );
                  }).toList(),
                ),
              ],

              Expanded(child: navigationShell),
            ],
          ),
        );
      },
    );
  }
}
