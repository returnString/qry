import * as qry from 'qry-web';

const interpreter = new qry.Interpreter()
const editor = document.getElementById('editor')
document.getElementById('eval').onclick = () => interpreter.eval(editor.value)
