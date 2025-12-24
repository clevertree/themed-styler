import React from 'react'
import { TSDiv } from './components/TSDiv'

export const SafeAreaView = (props: any) => <TSDiv tag="main" {...props} />
export const ScrollView = (props: any) => <TSDiv tag="div" {...props} />
export const Text = (props: any) => <TSDiv tag="span" {...props} />
export const TextInput = (props: any) => <TSDiv tag="input" {...props} />
export const TouchableOpacity = (props: any) => <TSDiv tag="button" {...props} />
export const View = (props: any) => <TSDiv tag="div" {...props} />
