import { TurboModuleRegistry, NativeModules } from 'react-native';
console.log('[NativeThemedStyler] Available NativeModules:', Object.keys(NativeModules));
console.log('[NativeThemedStyler] Has ThemedStyler in NativeModules?', !!NativeModules.ThemedStyler);
console.log('[NativeThemedStyler] TurboModule result:', TurboModuleRegistry.get('ThemedStyler'));
export default TurboModuleRegistry.get('ThemedStyler') ?? NativeModules.ThemedStyler;
//# sourceMappingURL=NativeThemedStyler.js.map