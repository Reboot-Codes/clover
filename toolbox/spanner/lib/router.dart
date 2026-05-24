import 'package:go_router/go_router.dart';

import 'package:spanner/features/welcome/screens/landing.dart';

import 'package:spanner/features/settings/components/shell.dart';
import 'package:spanner/features/settings/screens/categories.dart';

import 'features/configurator/components/shell.dart';
import 'features/configurator/screens/overview.dart';
import 'features/configurator/screens/modules/overview.dart';
import 'features/configurator/screens/gestures/quick_settings.dart';
import 'features/configurator/screens/apps/apps_list.dart';
import 'features/configurator/screens/repos/repo_list.dart';

import 'features/wizard/components/shell.dart';
import 'features/wizard/screens/get_started.dart';

// GoRouter configuration
final router = GoRouter(
  initialLocation: '/',
  routes: <RouteBase>[
    StatefulShellRoute.indexedStack(
      builder: (context, state, navigationShell) {
        return ConfiguratorShell(navigationShell: navigationShell);
      },
      branches: [
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/configurator',
              builder: (context, state) => const ConfiguratorOverview(),
            ),
          ],
        ),
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/configurator/modules',
              builder: (context, state) => const ModulesOverview(),
            ),
          ],
        ),
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/configurator/gestures',
              builder: (context, state) => const GestureQuickSettings(),
            ),
          ],
        ),
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/configurator/apps',
              builder: (context, state) => const AppsList(),
            ),
          ],
        ),
        StatefulShellBranch(
          routes: [
            GoRoute(
              path: '/configurator/repos',
              builder: (context, state) => const RepoList(),
            ),
          ],
        ),
      ],
    ),
    ShellRoute(
      builder: (context, state, child) {
        return WizardShell(child: child);
      },
      routes: [
        GoRoute(
          path: "/wizard",
          builder: (context, state) => const WizardGetStarted(),
        ),
      ],
    ),
    // TODO: Move to ShellRoute for desktop/tablet optimization
    ShellRoute(
      builder: (context, state, child) {
        return SettingsShell(child: child);
      },
      routes: [
        GoRoute(
          path: "/settings",
          builder: (context, state) => const SettingsCategories(),
        ),
      ],
    ),
    GoRoute(path: "/", builder: (context, state) => const LandingPage()),
  ],
);
